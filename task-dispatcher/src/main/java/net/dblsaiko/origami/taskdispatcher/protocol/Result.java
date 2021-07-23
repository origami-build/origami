package net.dblsaiko.origami.taskdispatcher.protocol;

public sealed interface Result<T, E> permits Result.Ok, Result.Err {
    final record Ok<T, E>(T val) implements Result<T, E> {
    }

    final record Err<T, E>(E error) implements Result<T, E> {
    }

    static <T, E> Result.Ok<T, E> ok(T ok) {
        return new Ok<>(ok);
    }

    static <T, E> Result.Err<T, E> err(E err) {
        return new Err<>(err);
    }

    static <T, E> BinDeserializeFor<Result<T, E>> deserialize(BinDeserializeFor<T> desT, BinDeserializeFor<E> desE) {
        return deserializer -> {
            int variant = deserializer.readU8();

            return switch (variant) {
                case 0 -> Result.ok(desT.deserialize(deserializer));
                case 1 -> Result.err(desE.deserialize(deserializer));
                default -> throw new IllegalStateException("Invalid Result variant with id %d".formatted(variant));
            };
        };
    }

    static <T, E> BinSerializeFor<Result<T, E>> serialize(BinSerializeFor<T> serT, BinSerializeFor<E> serE) {
        return (obj, serializer) -> {
            if (obj instanceof Ok<T, E> ok) {
                serializer.writeU8(0);
                serT.serialize(ok.val(), serializer);
            } else if (obj instanceof Err<T, E> err) {
                serializer.writeU8(1);
                serE.serialize(err.error(), serializer);
            } else {
                throw new IllegalStateException("unreachable");
            }
        };
    }
}
