package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public final record ExecResult(int tag, Result<TaskInfo, ExecError> result) implements BinSerialize {
    private static final BinSerializeFor<Result<TaskInfo, ExecError>> RESULT_SERIALIZER = Result.serialize(TaskInfo::serialize, ExecError::serialize);

    @Override
    public void serialize(BinSerializer serializer) throws IOException {
        serializer.writeU32(this.tag);
        RESULT_SERIALIZER.serialize(this.result, serializer);
    }
}
