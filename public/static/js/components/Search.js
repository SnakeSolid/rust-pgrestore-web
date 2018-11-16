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
		const res = reqwest({
			url: "/api/v1/search",
			type: "json",
  			method: "POST",
  			contentType: "application/json",
  			data: JSON.stringify({
				query: this.query(),
			}),
		}).then(function(resp) {
			if (resp.success) {
				this.isError(false);
				this.results(resp.result);
			} else {
				this.isError(true);
				this.errorMessage(resp.message);
				this.results([]);
			}

			this.isLoading(false);
		}.bind(this)).fail(function(err, msg) {
			this.isLoading(false);
			this.isError(true);
			this.errorMessage(msg);
			this.results([]);
		}.bind(this));

		this.isLoading(true);
	};

	Restore.prototype.cancelClick = function() {
		this.visible(false);
	};

	return Restore;
});
