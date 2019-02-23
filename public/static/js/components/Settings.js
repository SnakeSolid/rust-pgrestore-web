"use strict";

define(["knockout", "Storage"], function(ko, Storage) {
	const Settings = function(params) {
		this.availableDestinations = params.destinations;
		this.selectedDestination = ko.observable();

		this.updateSelectedDestination();
		this.availableDestinations.subscribe(this.updateSelectedDestination);
	};

	Settings.prototype.updateSelectedDestination = function() {
		this.selectedDestination(Storage.getPreferredDestination());
	};

	Settings.prototype.saveSettings = function() {
		Storage.setPreferredDestination(this.selectedDestination());
	};

	return Settings;
});
