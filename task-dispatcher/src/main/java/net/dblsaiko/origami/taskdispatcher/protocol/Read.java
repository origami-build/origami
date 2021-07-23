package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public record Read(int tag, int stream, int size) implements BinSerialize {
    @Override
    public void serialize(BinSerializer serializer) throws IOException {
        serializer.writeU32(this.tag);
        serializer.writeU32(this.stream);
        serializer.writeU32(this.size);
    }
}
