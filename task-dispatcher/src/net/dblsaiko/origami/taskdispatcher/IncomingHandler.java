package net.dblsaiko.origami.taskdispatcher;

import java.time.Duration;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.concurrent.CompletableFuture;

import net.dblsaiko.origami.taskdispatcher.protocol.ExecError;
import net.dblsaiko.origami.taskdispatcher.protocol.Result;
import net.dblsaiko.origami.taskdispatcher.protocol.TaskInfo;

public interface IncomingHandler {
    Result<TaskInfo, ExecError> exec(String mainClass, String[] params, OptionalInt stdout, OptionalInt stderr, OptionalInt stdin);

    CompletableFuture<Boolean> wait(int taskId, Optional<Duration> timeout);
}
