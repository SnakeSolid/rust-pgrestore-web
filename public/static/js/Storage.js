"use strict";

define(["exports"], function(exports) {
	const KEY_PREFERRED_DESTINATION = "PreferredDestination";
	const KEY_NAME_PATTERNS = "NamePatterns";

	function read(key, defaultValue) {
		const result = localStorage.getItem(key);

		if (result === null) {
			return defaultValue;
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
		return read(KEY_PREFERRED_DESTINATION, undefined);
	};

	exports.setPreferredDestination = function(value) {
		if (value !== undefined) {
			write(KEY_PREFERRED_DESTINATION, value);
		} else {
			remove(KEY_PREFERRED_DESTINATION);
		}
	};

	exports.getNamePatterns = function() {
		return read(KEY_NAME_PATTERNS, []);
	};

	exports.setNamePatterns = function(value) {
		if (value !== undefined) {
			write(KEY_NAME_PATTERNS, value);
		} else {
			remove(KEY_NAME_PATTERNS);
		}
	};
});
