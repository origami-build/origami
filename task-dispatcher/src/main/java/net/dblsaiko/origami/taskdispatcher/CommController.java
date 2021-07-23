package net.dblsaiko.origami.taskdispatcher;

import java.io.IOException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executor;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

import net.dblsaiko.origami.taskdispatcher.protocol.BinDeserializer;
import net.dblsaiko.origami.taskdispatcher.protocol.BinSerializer;
import net.dblsaiko.origami.taskdispatcher.protocol.Close;
import net.dblsaiko.origami.taskdispatcher.protocol.CloseResult;
import net.dblsaiko.origami.taskdispatcher.protocol.Exec;
import net.dblsaiko.origami.taskdispatcher.protocol.ExecError;
import net.dblsaiko.origami.taskdispatcher.protocol.ExecResult;
import net.dblsaiko.origami.taskdispatcher.protocol.FromJvm;
import net.dblsaiko.origami.taskdispatcher.protocol.IoError;
import net.dblsaiko.origami.taskdispatcher.protocol.Read;
import net.dblsaiko.origami.taskdispatcher.protocol.ReadResult;
import net.dblsaiko.origami.taskdispatcher.protocol.Result;
import net.dblsaiko.origami.taskdispatcher.protocol.TaskInfo;
import net.dblsaiko.origami.taskdispatcher.protocol.ToJvm;
import net.dblsaiko.origami.taskdispatcher.protocol.Unit;
import net.dblsaiko.origami.taskdispatcher.protocol.WaitResult;
import net.dblsaiko.origami.taskdispatcher.protocol.Write;
import net.dblsaiko.origami.taskdispatcher.protocol.WriteResult;

public class CommController {
    private final BinSerializer writer;
    private final BinDeserializer reader;
    private final AsyncQueue<WriteResult> writeResultQueue = new AsyncQueue<>();
    private final AsyncQueue<ReadResult> readResultQueue = new AsyncQueue<>();
    private final AsyncQueue<CloseResult> closeResultQueue = new AsyncQueue<>();
    private final Lock writeLock = new ReentrantLock();
    private int tag;

    public CommController(BinSerializer writer, BinDeserializer reader) {
        this.writer = writer;
        this.reader = reader;
    }

    public void send(FromJvm packet) throws IOException {
        Util.withLockIo(this.writeLock, () -> packet.serialize(this.writer));
    }

    private int getNextTag() {
        var tag = this.tag;
        this.tag += 1;
        return tag;
    }

    public CompletableFuture<Result<Integer, IoError>> write(int stream, byte[] data) throws IOException {
        var tag = this.getNextTag();
        CompletableFuture<WriteResult> fut = this.writeResultQueue.startCallback(tag);
        this.send(new FromJvm.Write(new Write(tag, stream, data)));
        return fut.thenApply(WriteResult::result);
    }

    public CompletableFuture<Result<byte[], IoError>> read(int stream, int size) throws IOException {
        var tag = this.getNextTag();
        CompletableFuture<ReadResult> fut = this.readResultQueue.startCallback(tag);
        this.send(new FromJvm.Read(new Read(tag, stream, size)));
        return fut.thenApply(ReadResult::result);
    }

    public CompletableFuture<Result<Unit, IoError>> close(int stream) throws IOException {
        var tag = this.getNextTag();
        CompletableFuture<CloseResult> fut = this.closeResultQueue.startCallback(tag);
        this.send(new FromJvm.Close(new Close(tag, stream)));
        return fut.thenApply(CloseResult::result);
    }

    public void readerLoop(IncomingHandler handler) throws IOException {
        while (true) {
            ToJvm msg = ToJvm.deserialize(this.reader);

            if (msg instanceof ToJvm.Exec exec) {
                Exec inner = exec.inner();
                Result<TaskInfo, ExecError> result = handler.exec(inner.mainClass(), inner.params(), inner.stdout(), inner.stderr(), inner.stdin());
                this.send(new FromJvm.ExecResult(new ExecResult(inner.tag(), result)));
            } else if (msg instanceof ToJvm.WriteResult writeResult) {
                this.writeResultQueue.finishCallback(writeResult.inner().tag(), writeResult.inner());
            } else if (msg instanceof ToJvm.ReadResult readResult) {
                this.readResultQueue.finishCallback(readResult.inner().tag(), readResult.inner());
            } else if (msg instanceof ToJvm.Wait wait) {
                CompletableFuture<Boolean> fut = handler.wait(wait.inner().task(), wait.inner().timeout());
                fut.thenAccept(timeout -> {
                    try {
                        this.send(new FromJvm.WaitResult(new WaitResult(wait.inner().tag(), timeout)));
                    } catch (IOException e) {
                        e.printStackTrace();
                    }
                });
            } else if (msg instanceof ToJvm.CloseResult closeResult) {
                this.closeResultQueue.finishCallback(closeResult.inner().tag(), closeResult.inner());
            }
        }
    }
}
