package net.dblsaiko.origami.taskdispatcher;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.locks.ReadWriteLock;
import java.util.concurrent.locks.ReentrantReadWriteLock;

import net.dblsaiko.origami.taskdispatcher.protocol.ExecError;
import net.dblsaiko.origami.taskdispatcher.protocol.Result;
import net.dblsaiko.origami.taskdispatcher.protocol.TaskInfo;

public class Dispatcher implements IncomingHandler {
    private final MuxOutputStream stdout;
    private final MuxOutputStream stderr;
    private final MuxInputStream stdin;
    private final CommController ctl;
    private final Map<String, Result<MethodHandle, ExecError>> entrypoints = new HashMap<>();

    private final Map<Integer, List<CompletableFuture<Boolean>>> waitingFutures = new HashMap<>();
    private final ReadWriteLock waitingFuturesLock = new ReentrantReadWriteLock();

    private final ScheduledExecutorService exec = Executors.newSingleThreadScheduledExecutor();

    private int taskId = 0;

    public Dispatcher(
        MuxOutputStream stdout,
        MuxOutputStream stderr,
        MuxInputStream stdin,
        CommController ctl
    ) {
        this.stdout = stdout;
        this.stderr = stderr;
        this.stdin = stdin;
        this.ctl = ctl;
    }

    @Override
    public Result<TaskInfo, ExecError> exec(String mainClassName, String[] params, OptionalInt stdout, OptionalInt stderr, OptionalInt stdin) {
        MethodHandle mh;

        if (!this.entrypoints.containsKey(mainClassName)) {
            Class<?> mainClass;

            try {
                mainClass = Class.forName(mainClassName);
            } catch (ClassNotFoundException e) {
                e.printStackTrace();
                ExecError err = new ExecError.InvalidClass(e.getMessage());
                this.entrypoints.put(mainClassName, Result.err(err));
                return Result.err(err);
            }

            try {
                mh = MethodHandles.lookup().findStatic(mainClass, "main", MethodType.methodType(void.class, String[].class));
            } catch (NoSuchMethodException | IllegalAccessException e) {
                e.printStackTrace();
                ExecError err = new ExecError.NoMainFn(e.getMessage());
                this.entrypoints.put(mainClassName, Result.err(err));
                return Result.err(err);
            }

            this.entrypoints.put(mainClassName, Result.ok(mh));
        } else {
            Result<MethodHandle, ExecError> mh1 = this.entrypoints.get(mainClassName);

            if (mh1 instanceof Result.Ok<MethodHandle, ExecError> ok) {
                mh = ok.val();
            } else if (mh1 instanceof Result.Err<MethodHandle, ExecError> err) {
                return Result.err(err.error());
            } else {
                throw new IllegalStateException("unreachable");
            }
        }

        OutputStream out;
        OutputStream err;
        InputStream in;

        if (stdout.isPresent()) {
            out = WrappedOutputStream.create(stdout.getAsInt(), this.ctl);
        } else {
            out = OutputStream.nullOutputStream();
        }

        if (stderr.isPresent()) {
            err = WrappedOutputStream.create(stderr.getAsInt(), this.ctl);
        } else {
            err = OutputStream.nullOutputStream();
        }

        if (stdin.isPresent()) {
            in = WrappedInputStream.create(stdin.getAsInt(), this.ctl);
        } else {
            in = InputStream.nullInputStream();
        }

        int taskId = this.taskId;
        this.taskId += 1;

        this.startProc(taskId, out, err, in, mh, params);

        return Result.ok(new TaskInfo(taskId));
    }

    private void startProc(int taskId, OutputStream out, OutputStream err, InputStream in, MethodHandle main, String[] args) {
        ThreadGroup tg = new ThreadGroup(String.format("Task %d", taskId));

        Util.withLock(this.waitingFuturesLock.writeLock(), () -> {
            this.waitingFutures.put(taskId, new ArrayList<>());
        });

        var thread = new Thread(tg, () -> {
            this.stdout.setThreadOutputStream(out);
            this.stderr.setThreadOutputStream(err);
            this.stdin.setThreadInputStream(in);

            try {
                // let's goooo
                // noinspection ConfusingArgumentToVarargsMethod
                main.invokeExact(args);

                this.taskExit(taskId);
            } catch (Throwable e) {
                this.taskFail(taskId, e);
            }

            this.stdout.setThreadOutputStream(null);
            this.stderr.setThreadOutputStream(null);

            try {
                out.close();
                err.close();
                in.close();
            } catch (IOException e) {
                e.printStackTrace();
            }
        }, String.format("Task %d: Main thread", taskId));
        thread.start();
    }

    private void taskExit(int taskId) {
        this.cleanFutures(taskId);
    }

    private void taskFail(int taskId, Throwable t) {
        this.cleanFutures(taskId);
    }

    private void cleanFutures(int taskId) {
        var futures = Util.withLock(this.waitingFuturesLock.writeLock(), () -> this.waitingFutures.remove(taskId));

        for (CompletableFuture<Boolean> future : futures) {
            future.complete(false);
        }
    }

    @Override
    public CompletableFuture<Boolean> wait(int taskId, Optional<Duration> timeout) {
        CompletableFuture<Boolean> future = new CompletableFuture<>();

        Util.withLock(this.waitingFuturesLock.writeLock(), () -> {
            List<CompletableFuture<Boolean>> futures = this.waitingFutures.get(taskId);

            if (futures != null) {
                futures.add(future);

                timeout.ifPresent(duration -> {
                    this.exec.schedule(() -> {
                        Util.withLock(this.waitingFuturesLock.writeLock(), () -> {
                            var futures1 = this.waitingFutures.get(taskId);

                            if (futures1 != null) {
                                futures1.remove(future);
                            }
                        });

                        future.complete(true);
                    }, duration.toMillis(), TimeUnit.MILLISECONDS);
                });
            } else {
                future.complete(false);
            }
        });

        return future;
    }
}
