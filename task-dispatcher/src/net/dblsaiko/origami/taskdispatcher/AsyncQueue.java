package net.dblsaiko.origami.taskdispatcher;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public final class AsyncQueue<T> {
    private final Map<Integer, CompletableFuture<T>> queue = new HashMap<>();
    private final Lock queueLock = new ReentrantLock();

    public CompletableFuture<T> startCallback(int tag) {
        CompletableFuture<T> fut = new CompletableFuture<>();

        Util.withLock(this.queueLock, () -> {
            this.queue.put(tag, fut);
        });

        return fut;
    }

    public void finishCallback(int tag, T msg) {
        CompletableFuture<T> fut = Util.withLock(this.queueLock, () -> this.queue.remove(tag));

        fut.complete(msg);
    }
}
