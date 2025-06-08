class Loader {
  dispatchList = [];
  itemsToLoad = 0;
  results = {};

  constructor() {
    
  }

  loadTo(v, func, ...args) {
    this.dispatchList.push([v, func, args]);
  }

  start(callback, onError) {
    // Setup callbacks.
    if (callback === undefined) {
      this._onFinish = function (results) {
        for (const [key, value] of Object.entries(results)) {
          window.global.var[key] = value;
        }
      }
    } else {
      this._onFinish = callback;
    }

    if (onError === undefined) {
      this._onError = function (name, err) {
        console.error("Encountered an error when attempting to load " + name);
        console.error(err);
      }
    } else {
      this._onError = onError;
    }

    // Start loading items.
    this.itemsToLoad = this.dispatchList.length;
    for (var i = 0; i < this.dispatchList.length; ++i) {
      var item = this.dispatchList[i];
      var func = item[1];
      var args = item[2];
      // Need to be let for the closure scoping!
      let name = item[0];
      var onSuccess = function(result) { this._onLoad(name, result); }.bind(this)
      var onError = function(err) { this._onError(name, err); }.bind(this)
      func(...args, onSuccess, onError);
    }
  }

  _onLoad(name, result) {
    this.results[str(name)] = result;
    this.itemsToLoad -= 1;
    if (this.itemsToLoad === 0) {
      this._onFinish(this.results);
    }
  }
}


