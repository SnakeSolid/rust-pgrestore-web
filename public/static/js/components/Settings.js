"use strict";

define(["knockout", "reqwest", "Storage"], function(ko, reqwest, Storage) {
	const Settings = function(params) {
		this.availableDestinations = ko.observableArray();
		this.selectedDestination = ko.observable();
		this.isLoading = ko.observable(false);
		this.isError = ko.observable(false);
		this.errorMessage = ko.observable();

		// TODO: Remove common destinations code with restore.
		this.loadDestinations();
	};

	Settings.prototype.loadDestinations = function() {
		const res = reqwest({
			url: "/api/v1/destination",
			type: "json",
			method: "POST",
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.availableDestinations(resp.result);
						this.selectedDestination(Storage.getPreferredDestination());
						this.isError(false);
					} else {
						this.isError(true);
						this.errorMessage(resp.message);
					}

					this.isLoading(false);
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.isLoading(false);
					this.isError(true);
					this.errorMessage(msg || err.responseText);
				}.bind(this)
			);

		this.isLoading(true);
	};

	Settings.prototype.saveSettings = function() {
		Storage.setPreferredDestination(this.selectedDestination());
	};

	return Settings;
});
