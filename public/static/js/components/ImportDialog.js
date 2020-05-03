"use strict";

define(["knockout", "Storage"], function(ko, Storage) {
	const Restore = function(params) {
		this.callback = params.callback;
		this.visible = params.visible;
		this.text = ko.observable("");
		this.errorText = ko.observable("");

		this.isErrorVisible = ko.pureComputed(function() {
			return this.errorText() !== "";
		}, this);
	};

	Restore.prototype.importClick = function() {
		const text = this.text();
		const semicolonIndex = text.indexOf(";");

		if (semicolonIndex === -1) {
			this.errorText("Invalid settings format. Semicolon must separate fields.");

			return;
		}

		const selectedDestination = parseInt(text.substring(0, semicolonIndex));
		let namePatterns;

		if (isNaN(selectedDestination)) {
			this.errorText("First field must contain integer number.");

			return;
		}

		if (selectedDestination < 0) {
			this.errorText("First field must contain positive number.");

			return;
		}

		try {
			namePatterns = JSON.parse(text.substring(semicolonIndex + 1));
		} catch (e) {
			this.errorText("Second field must contain valid JSON array: " + e);

			return;
		}

		Storage.setPreferredDestination(selectedDestination);
		Storage.setNamePatterns(namePatterns);

		this.errorText("");
		this.visible(false);
		this.callback();
	};

	Restore.prototype.closeClick = function() {
		this.visible(false);
	};

	return Restore;
});
