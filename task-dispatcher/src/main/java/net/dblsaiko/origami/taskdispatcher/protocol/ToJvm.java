package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public sealed interface ToJvm permits ToJvm.Exec, ToJvm.ReadResult, ToJvm.Wait, ToJvm.WriteResult, ToJvm.CloseResult {
    final record Exec(net.dblsaiko.origami.taskdispatcher.protocol.Exec inner) implements ToJvm {
    }

    final record WriteResult(net.dblsaiko.origami.taskdispatcher.protocol.WriteResult inner) implements ToJvm {
    }

    final record ReadResult(net.dblsaiko.origami.taskdispatcher.protocol.ReadResult inner) implements ToJvm {
    }

    final record Wait(net.dblsaiko.origami.taskdispatcher.protocol.Wait inner) implements ToJvm {
    }

    final record CloseResult(net.dblsaiko.origami.taskdispatcher.protocol.CloseResult inner) implements ToJvm {
    }

    static ToJvm deserialize(BinDeserializer deserializer) throws IOException {
        int variant = deserializer.readUsize();

        return switch (variant) {
            case 0 -> new Exec(net.dblsaiko.origami.taskdispatcher.protocol.Exec.deserialize(deserializer));
            case 1 -> new WriteResult(net.dblsaiko.origami.taskdispatcher.protocol.WriteResult.deserialize(deserializer));
            case 2 -> new ReadResult(net.dblsaiko.origami.taskdispatcher.protocol.ReadResult.deserialize(deserializer));
            case 3 -> new Wait(net.dblsaiko.origami.taskdispatcher.protocol.Wait.deserialize(deserializer));
            case 4 -> new CloseResult(net.dblsaiko.origami.taskdispatcher.protocol.CloseResult.deserialize(deserializer));
            default -> throw new IllegalStateException("Invalid ToJvm variant %d".formatted(variant));
        };
    }
}
