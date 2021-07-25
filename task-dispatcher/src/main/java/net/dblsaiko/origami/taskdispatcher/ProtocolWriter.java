package net.dblsaiko.origami.taskdispatcher;

import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;

import net.dblsaiko.origami.taskdispatcher.protocol.BinSerializer;

public class ProtocolWriter implements BinSerializer {
    private final OutputStream stream;
    private final OutputStream fileOut;

    public ProtocolWriter(OutputStream stream) {
        this.stream = stream;

        if (!System.getenv().getOrDefault("ORIGAMI_PROTO_DEBUG", "").isBlank()) {
            try {
                this.fileOut = new FileOutputStream("out.bin");
            } catch (FileNotFoundException e) {
                e.printStackTrace();
                throw new RuntimeException(e);
            }
        } else {
            this.fileOut = null;
        }
    }

    @Override
    public void writeString(String s) throws IOException {
        this.writeVarInt(s.length());
        this.stream.write(s.getBytes(StandardCharsets.UTF_8));
        this.stream.flush();

        if (this.fileOut != null) {
            this.fileOut.write(s.getBytes(StandardCharsets.UTF_8));
            this.fileOut.flush();
        }
    }

    @Override
    public void writeU8(int u8) throws IOException {
        this.stream.write(u8);
        this.stream.flush();

        if (this.fileOut != null) {
            this.fileOut.write(u8);
            this.fileOut.flush();
        }
    }

    @Override
    public void writeU32(int u32) throws IOException {
        this.stream.write(u32);
        this.stream.write(u32 >> 8);
        this.stream.write(u32 >> 16);
        this.stream.write(u32 >> 24);
        this.stream.flush();

        if (this.fileOut != null) {
            this.fileOut.write(u32);
            this.fileOut.write(u32 >> 8);
            this.fileOut.write(u32 >> 16);
            this.fileOut.write(u32 >> 24);
            this.fileOut.flush();
        }
    }

    @Override
    public void writeUsize(int num) throws IOException {
        this.writeVarInt(num);
    }

    @Override
    public void writeBool(boolean bool) throws IOException {
        this.writeU8(bool ? -1 : 0);
    }

    private void writeVarInt(int num) throws IOException {
        var numPos = 0;
        var idx = 0;
        var buf = new byte[5];
        var dataBits = 32 - Integer.numberOfLeadingZeros(num);

        do {
            byte next;

            if (dataBits - numPos > 7) {
                next = (byte) 0b10000000;
            } else {
                next = 0;
            }

            var piece = (byte) ((num >> numPos) & 0b01111111 | next);
            buf[idx] = piece;
            idx += 1;
            numPos += 7;
        } while (numPos < dataBits);

        this.stream.write(buf, 0, idx);
        this.stream.flush();

        if (this.fileOut != null) {
            this.fileOut.write(buf, 0, idx);
            this.fileOut.flush();
        }
    }

}
