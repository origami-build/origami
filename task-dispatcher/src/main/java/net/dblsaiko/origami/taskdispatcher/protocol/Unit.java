package net.dblsaiko.origami.taskdispatcher.protocol;

public enum Unit implements BinSerialize {
    INSTANCE;

    @Override
    public String toString() {
        return "Unit";
    }

    @Override
    public void serialize(BinSerializer serializer) {
        // serializes to nothing
    }

    public static Unit deserialize(BinDeserializer deserializer) {
        return Unit.INSTANCE;
    }
}
