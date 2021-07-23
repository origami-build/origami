package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public final record Write(int tag, int stream, byte[] data) implements BinSerialize {
    @Override
    public void serialize(BinSerializer serializer) throws IOException {
        serializer.writeU32(this.tag);
        serializer.writeU32(this.stream);
        BinSerializeFor.BYTES.serialize(this.data, serializer);
    }

    @Override
    public String toString() {
        return "Write[tag=%d, stream=%d, data=[%d bytes]".formatted(this.tag, this.stream, this.data.length);
    }
}
