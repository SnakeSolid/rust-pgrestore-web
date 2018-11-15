"use strict";

define([ "knockout", "reqwest" ], function(ko, reqwest) {
	const Restore = function(params) {
		this.callback = params.callback;


		this.query = ko.observable("");
		this.isLoading = ko.observable(false);
		this.isError = ko.observable(false);
		this.errorMessage = ko.observable("");
		this.results = ko.observableArray([]);

		this.hasReults = ko.pureComputed(function() {
			return this.results().length > 0;
		}, this);

		this.selectResult = function(value) {
			this.callback(value);
		}.bind(this);
	};

	Restore.prototype.findBackups = function() {
		const self = this;
		const res = reqwest({
			url: "/api/v1/search",
			type: "json",
  			method: "POST",
  			contentType: "application/json",
  			data: JSON.stringify({
				query: self.query(),
			}),
		}).then(function(resp) {
			if (resp.success) {
				self.results(resp.result);
				self.isError(false);
			} else {
				self.isError(true);
				self.errorMessage(resp.message);
			}

			self.isLoading(false);
		}).fail(function(err, msg) {
			self.isLoading(false);
			self.isError(true);
			self.errorMessage(msg);
		});

		this.isLoading(true);
	};

	Restore.prototype.cancelClick = function() {
		this.visible(false);
	};

	return Restore;
});
