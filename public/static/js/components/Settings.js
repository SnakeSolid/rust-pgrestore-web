"use strict";

define(["knockout", "Storage"], function(ko, Storage) {
	const Settings = function(params) {
		this.availableDestinations = params.destinations;
		this.selectedDestination = ko.observable();
		this.namePatterns = ko.observableArray();
		this.editPathPattern = ko.observable();
		this.editReplacePattern = ko.observable();
		this.editChangeCase = ko.observable();

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

		this.isNoChangeCase = function(item) {
			return item.changeCase === "NoChange";
		}.bind(this);

		this.isUppercase = function(item) {
			return item.changeCase === "Upper";
		}.bind(this);

		this.isLowercase = function(item) {
			return item.changeCase === "Lower";
		}.bind(this);

		this.updateSelectedDestination();
		this.updateNamePatterns();

		this.exportVisible = ko.observable(false);
		this.availableDestinations.subscribe(this.updateSelectedDestination);
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

	Settings.prototype.showExport = function() {
		this.exportVisible(true);
	};

	return Settings;
});
