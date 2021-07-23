package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public final record Close(int tag, int stream) implements BinSerialize {
    @Override
    public void serialize(BinSerializer serializer) throws IOException {
        serializer.writeU32(this.tag);
        serializer.writeU32(this.stream);
    }
}
