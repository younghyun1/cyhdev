// Added by the website: this map runs in an opaque-origin sandbox, where reading
// window.localStorage throws. Provide a bounded in-memory replacement so layer
// toggles work for the current page view.
(function () {
  try {
    window.localStorage.getItem("cyhdev-storage-probe");
    return;
  } catch (unavailable) {
    // Fall through to the in-memory replacement.
  }
  var MAX_KEYS = 256;
  var values = new Map();
  var storage = {
    get length() {
      return values.size;
    },
    key: function (index) {
      var keys = Array.from(values.keys());
      return index >= 0 && index < keys.length ? keys[index] : null;
    },
    getItem: function (key) {
      key = String(key);
      return values.has(key) ? values.get(key) : null;
    },
    setItem: function (key, value) {
      key = String(key);
      if (!values.has(key) && values.size >= MAX_KEYS) {
        throw new DOMException("Storage limit reached", "QuotaExceededError");
      }
      values.set(key, String(value));
    },
    removeItem: function (key) {
      values.delete(String(key));
    },
    clear: function () {
      values.clear();
    },
  };
  try {
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      enumerable: true,
      value: storage,
    });
  } catch (unsupported) {
    // Leave the original accessor; squaremap overlays may fail to load.
  }
})();
