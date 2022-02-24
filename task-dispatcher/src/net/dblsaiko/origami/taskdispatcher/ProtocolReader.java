package net.dblsaiko.origami.taskdispatcher;

import java.io.EOFException;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

import net.dblsaiko.origami.taskdispatcher.protocol.BinDeserializer;

public class ProtocolReader implements BinDeserializer {
    private final InputStream stream;

    public ProtocolReader(InputStream stream) {
        this.stream = stream;
    }

    @Override
    public int readU8() throws IOException {
        int b = this.stream.read();
        if (b == -1) throw new EOFException();
        return b;
    }

    @Override
    public int readU32() throws IOException {
        byte[] bytes = this.stream.readNBytes(4);
        return (bytes[0] & 0xFF) |
            (bytes[1] & 0xFF) << 8 |
            (bytes[2] & 0xFF) << 16 |
            (bytes[3] & 0xFF) << 24;
    }

    @Override
    public long readU64() throws IOException {
        byte[] bytes = this.stream.readNBytes(8);
        return (bytes[0] & 0xFFL) |
            (bytes[1] & 0xFFL) << 8 |
            (bytes[2] & 0xFFL) << 16 |
            (bytes[3] & 0xFFL) << 24 |
            (bytes[4] & 0xFFL) << 32 |
            (bytes[5] & 0xFFL) << 40 |
            (bytes[6] & 0xFFL) << 48 |
            (bytes[7] & 0xFFL) << 56;
    }

    @Override
    public String readString() throws IOException {
        int length = this.readUsize();
        byte[] b = this.stream.readNBytes(length);
        return new String(b, StandardCharsets.UTF_8);
    }

    @Override
    public int readUsize() throws IOException {
        var offset = 0;
        var num = 0;

        while (true) {
            var b = this.readU8();
            var has_next = (b & 0b10000000) != 0;
            num |= (b & 0b01111111) << offset;
            offset += 7;

            if (!has_next) {
                break;
            }
        }

        return num;
    }
}
