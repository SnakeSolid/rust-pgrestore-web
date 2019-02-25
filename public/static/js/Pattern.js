"use strict";

define(["Storage", "exports"], function(Storage, exports) {
	const CASE_NO_CHANGE = "NoChange";
	const CASE_TO_UPPER = "Upper";
	const CASE_TO_LOWER = "Lower";

	const sameCase = function(value) {
		return value;
	};

	const upperCase = function(value) {
		return value.toUpperCase();
	};

	const lowerCase = function(value) {
		return value.toLowerCase();
	};

	const changeCaseFn = function(changeCase) {
		if (changeCase === CASE_TO_UPPER) {
			return upperCase;
		} else if (changeCase === CASE_TO_LOWER) {
			return lowerCase;
		} else {
			return sameCase;
		}
	};

	const tryInferName = function(path, pathPattern, nameTemplate, changeCase) {
		const pattern = new RegExp(pathPattern);
		const match = path.match(pattern);

		if (match !== null) {
			return nameTemplate.replace(/\$\d+/g, function(group) {
				const index = parseInt(group.substring(1));
				const result = match[index];

				if (result === undefined) {
					return "";
				} else {
					return changeCase(result);
				}
			});
		} else {
			return undefined;
		}
	};

	exports.inferDatabaseName = function(backupPath) {
		const paterns = Storage.getNamePatterns();
		let result = undefined;

		if (paterns !== undefined) {
			for (const pattern of paterns) {
				const databaseName = tryInferName(
					backupPath,
					pattern.pathPattern,
					pattern.replacePattern,
					changeCaseFn(pattern.changeCase)
				);

				if (databaseName !== undefined) {
					result = databaseName;

					break;
				}
			}
		}

		return result;
	};
});
