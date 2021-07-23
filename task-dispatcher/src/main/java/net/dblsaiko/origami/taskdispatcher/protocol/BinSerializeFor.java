package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;
import java.util.Optional;
import java.util.OptionalInt;

@FunctionalInterface
public interface BinSerializeFor<T> {
    static <T> BinSerializeFor<Optional<T>> option(BinSerializeFor<T> serT) {
        return (t, serializer) -> {
            if (t.isEmpty()) {
                serializer.writeUsize(0);
            } else {
                serializer.writeUsize(1);
                serT.serialize(t.get(), serializer);
            }
        };
    }

    BinSerializeFor<Integer> U32 = (t, serializer) -> serializer.writeU32(t);

    BinSerializeFor<Integer> USIZE = (t, serializer) -> serializer.writeUsize(t);

    BinSerializeFor<OptionalInt> OPTION_U32 = (t, serializer) -> {
        if (t.isEmpty()) {
            serializer.writeUsize(0);
        } else {
            serializer.writeUsize(1);
            serializer.writeU32(t.getAsInt());
        }
    };

    BinSerializeFor<byte[]> BYTES = (t, serializer) -> {
        serializer.writeUsize(t.length);

        for (byte b : t) {
            serializer.writeU8(b);
        }
    };

    void serialize(T t, BinSerializer serializer) throws IOException;
}
