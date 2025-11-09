// index.js (compat shim)
// Re-export the new implementation in `index2.js` so existing consumers who
// import `index.js` keep working. The legacy implementation was removed and
// moved to `index2.js` which contains the canonical, tested API.

const isNode = typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

if (isNode) {
  // In Node.js environments re-export everything from index2.js
  module.exports = require('./index2.js');
} else {
  // In browsers assume `index2.js` is loaded alongside this file and has
  // populated `window.webGL2`. Nothing else to do here.
  // (Keeping a no-op shim avoids duplicating the implementation.)
}
