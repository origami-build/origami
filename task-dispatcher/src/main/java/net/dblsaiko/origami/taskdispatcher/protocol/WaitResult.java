package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public final record WaitResult(int tag, boolean timeout) implements BinSerialize {
    @Override
    public void serialize(BinSerializer serializer) throws IOException {
        serializer.writeU32(this.tag);
        serializer.writeBool(this.timeout);
    }
}
