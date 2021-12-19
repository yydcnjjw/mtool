var cache_storage_key = 'pwa'

var cache_list = [
    "index.html",
]

self.addEventListener('install', (e) => {
    console.info('Installing Service Worker');
    e.waitUntil(
        caches.open(cache_storage_key)
            .then(cache => cache.addAll(cache_list))
            .then(() => self.skipWaiting())
    )
});

self.addEventListener('fetch', function(e) {
  e.respondWith(
    caches.match(e.request).then(function(response) {
      if (response != null) {
        return response
      }
      return fetch(e.request.url)
    })
  )
})

self.addEventListener('activate', function(e) {

 event.waitUntil(
    caches.keys().then(function(cacheNames) {
      return Promise.all(
        cacheNames.map(function(cacheName) {
          if (cache_list.indexOf(cacheName) === -1) {
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
})

