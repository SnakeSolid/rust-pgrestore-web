"use strict";

define(["knockout", "Storage"], function(ko, Storage) {
	const Restore = function(params) {
		this.visible = params.visible;
		this.text = ko.observable();

		this.visible.subscribe(this.updateText.bind(this));
	};

	Restore.prototype.updateText = function(visibleValue) {
		if (visibleValue) {
			const preferredDestination = JSON.stringify(Storage.getPreferredDestination());
			const namePaterns = JSON.stringify(Storage.getNamePatterns());

			this.text(`${preferredDestination};${namePaterns}`);
		}
	};

	Restore.prototype.closeClick = function() {
		this.visible(false);
	};

	return Restore;
});
