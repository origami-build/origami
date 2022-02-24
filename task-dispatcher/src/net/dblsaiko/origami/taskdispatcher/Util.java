package net.dblsaiko.origami.taskdispatcher;

import java.io.IOException;
import java.util.concurrent.locks.Lock;
import java.util.function.Supplier;

public class Util {
    public static void withLock(Lock lock, Runnable op) {
        try {
            lock.lock();
            op.run();
        } finally {
            lock.unlock();
        }
    }

    public static void withLockIo(Lock lock, IoRunnable op) throws IOException {
        try {
            lock.lock();
            op.run();
        } finally {
            lock.unlock();
        }
    }

    public static <T> T withLock(Lock lock, Supplier<T> op) {
        try {
            lock.lock();
            return op.get();
        } finally {
            lock.unlock();
        }
    }

    @FunctionalInterface
    interface IoRunnable {
        void run() throws IOException;
    }
}
