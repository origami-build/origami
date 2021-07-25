package net.dblsaiko.origami.ojavac;

import java.util.Iterator;
import java.util.NoSuchElementException;

public class SliceIter<T> implements Iterable<T> {
    private final T[] array;
    private final int start;
    private final int endExclusive;

    private SliceIter(T[] array, int start, int endExclusive) {
        this.array = array;
        this.start = start;
        this.endExclusive = endExclusive;
    }

    public static <T> SliceIter<T> of(T[] array, int start, int endExclusive) {
        return new SliceIter<>(array, start, endExclusive);
    }

    @Override
    public Iterator<T> iterator() {
        return new Iterator<>() {
            private int pos = SliceIter.this.start;

            @Override
            public boolean hasNext() {
                return this.pos < SliceIter.this.endExclusive;
            }

            @Override
            public T next() {
                if (this.hasNext()) {
                    T t = SliceIter.this.array[this.pos];
                    this.pos += 1;
                    return t;
                } else {
                    throw new NoSuchElementException();
                }
            }
        };
    }
}
