"use strict";

define(["exports"], function(exports) {
	const KEY_PREFERRED_DESTINATION = "PreferredDestination";

	function read(key) {
		const result = localStorage.getItem(key);

		if (result === null) {
			return undefined;
		} else {
			return JSON.parse(result);
		}
	}

	function write(key, value) {
		localStorage.setItem(key, JSON.stringify(value));
	}

	function remove(key) {
		localStorage.removeItem(key);
	}

	exports.getPreferredDestination = function() {
		return read(KEY_PREFERRED_DESTINATION);
	};

	exports.setPreferredDestination = function(value) {
		if (value !== undefined) {
			write(KEY_PREFERRED_DESTINATION, value);
		} else {
			remove(KEY_PREFERRED_DESTINATION);
		}
	};
});
