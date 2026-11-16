const v8 = require("v8");
setInterval(() => {
  const m = process.memoryUsage();
  const h = v8.getHeapStatistics();
  console.error(
    JSON.stringify({
      th: require("worker_threads").threadId,
      t: Math.round(process.uptime()),
      rss: m.rss >> 20,
      heapTotal: m.heapTotal >> 20,
      heapUsed: m.heapUsed >> 20,
      external: m.external >> 20,
      ab: m.arrayBuffers >> 20,
      malloced: h.malloced_memory >> 20,
      peakMalloced: h.peak_malloced_memory >> 20,
    }),
  );
}, 8000).unref();
