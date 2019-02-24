"use strict";

define(["knockout", "Storage"], function(ko, Storage) {
	const Settings = function(params) {
		this.availableDestinations = params.destinations;
		this.selectedDestination = ko.observable();
		this.namePatterns = ko.observableArray();
		this.editPathPattern = ko.observable();
		this.editReplacePattern = ko.observable();
		this.editChangeCase = ko.observable();

		this.updateSelectedDestination();
		this.availableDestinations.subscribe(this.updateSelectedDestination);

		this.updateNamePatterns();

		this.moveUp = function(item) {
			const idx = this.namePatterns.indexOf(item);
			const patterns = this.namePatterns();
			const it = patterns[idx];
			const other = patterns[idx - 1];

			patterns[idx] = other;
			patterns[idx - 1] = it;

			this.namePatterns(patterns);
		}.bind(this);

		this.moveDown = function(item) {
			const idx = this.namePatterns.indexOf(item);
			const patterns = this.namePatterns();
			const it = patterns[idx];
			const other = patterns[idx + 1];

			patterns[idx] = other;
			patterns[idx + 1] = it;

			this.namePatterns(patterns);
		}.bind(this);

		this.removePattern = function(item) {
			this.namePatterns.remove(item);
		}.bind(this);
	};

	Settings.prototype.updateSelectedDestination = function() {
		this.selectedDestination(Storage.getPreferredDestination());
	};

	Settings.prototype.updateNamePatterns = function() {
		this.namePatterns(Storage.getNamePatterns());
	};

	Settings.prototype.isMoveUpAllowed = function(index) {
		return index() > 0;
	};

	Settings.prototype.isMoveDownAllowed = function(index) {
		return index() < this.namePatterns().length - 1;
	};

	Settings.prototype.addPattern = function() {
		this.namePatterns.push({
			pathPattern: this.editPathPattern(),
			replacePattern: this.editReplacePattern(),
			changeCase: this.editChangeCase(),
		});
	};

	Settings.prototype.saveSettings = function() {
		Storage.setPreferredDestination(this.selectedDestination());
		Storage.setNamePatterns(this.namePatterns());
	};

	return Settings;
});
