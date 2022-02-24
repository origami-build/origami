package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public final record TaskInfo(int taskId) implements BinSerialize {
    @Override
    public void serialize(BinSerializer serializer) throws IOException {
        serializer.writeU32(this.taskId);
    }
}
