package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.function.IntFunction;

@FunctionalInterface
public interface BinDeserializeFor<T> {
    static <T> BinDeserializeFor<Optional<T>> option(BinDeserializeFor<T> desT) {
        return deserializer -> {
            int variant = deserializer.readUsize();
            return switch (variant) {
                case 0 -> Optional.empty();
                case 1 -> Optional.of(desT.deserialize(deserializer));
                default -> throw new IllegalStateException("Invalid Option variant %d".formatted(variant));
            };
        };
    }

    BinDeserializeFor<OptionalInt> OPTION_U32 = deserializer -> {
        int variant = deserializer.readUsize();
        return switch (variant) {
            case 0 -> OptionalInt.empty();
            case 1 -> OptionalInt.of(deserializer.readU32());
            default -> throw new IllegalStateException("Invalid Option variant %d".formatted(variant));
        };
    };

    static <T> BinDeserializeFor<List<T>> list(BinDeserializeFor<T> desT) {
        return deserializer -> {
            int length = deserializer.readUsize();
            List<T> list = new ArrayList<>(length);

            for (int i = 0; i < length; i++) {
                list.add(desT.deserialize(deserializer));
            }

            return list;
        };
    }

    static <T> BinDeserializeFor<T[]> array(IntFunction<T[]> makeArray, BinDeserializeFor<T> desT) {
        return deserializer -> {
            int length = deserializer.readUsize();
            T[] array = makeArray.apply(length);

            for (int i = 0; i < length; i++) {
                array[i] = desT.deserialize(deserializer);
            }

            return array;
        };
    }

    BinDeserializeFor<byte[]> BYTES = deserializer -> {
        int length = deserializer.readUsize();
        byte[] bytes = new byte[length];

        for (int i = 0; i < length; i++) {
            bytes[i] = (byte) deserializer.readU8();
        }

        return bytes;
    };

    BinDeserializeFor<Duration> DURATION = deserializer -> {
        long secs = deserializer.readU64();
        int nanos = deserializer.readU32();
        return Duration.ofSeconds(secs, nanos & 0xFFFFFFFFL);
    };

    BinDeserializeFor<Integer> USIZE = BinDeserializer::readUsize;

    T deserialize(BinDeserializer deserializer) throws IOException;
}
