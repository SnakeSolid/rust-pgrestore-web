"use strict";

define(["knockout", "reqwest"], function(ko, reqwest) {
	const Restore = function(params) {
		this.callback = params.callback;
		this.visible = params.visible;

		this.text = ko.observable("");
	};

	Restore.prototype.cancelClick = function() {
		this.visible(false);
	};

	Restore.prototype.acceptClick = function() {
		this.visible(false);
		this.callback(this.text());
		this.text("");
	};

	return Restore;
});
