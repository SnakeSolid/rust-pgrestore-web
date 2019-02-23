"use strict";

define(["knockout", "reqwest", "components", "handlers"], function(ko, reqwest, _components, _handlers) {
	const PAGE_SEARCH = "Search";
	const PAGE_RESTORE = "Restore";
	const PAGE_STATUS = "Status";
	const PAGE_JOBS = "Jobs";
	const PAGE_SETTINGS = "Settings";

	const Application = function() {
		this.currentPage = ko.observable(PAGE_RESTORE);
		this.currentJobid = ko.observable();
		this.backupSearchResult = ko.observable("");
		this.destinations = ko.observableArray();

		this.isSearchVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_SEARCH;
		}, this);

		this.isRestoreVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_RESTORE;
		}, this);

		this.isStatusVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_STATUS;
		}, this);

		this.isJobsVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_JOBS;
		}, this);

		this.isSettingsVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_SETTINGS;
		}, this);

		this.searchCallback = function(value) {
			this.backupSearchResult(value);
			this.currentPage(PAGE_RESTORE);
		}.bind(this);

		this.showJStatusCallback = function(jobid) {
			this.currentJobid(jobid);
			this.currentPage(PAGE_STATUS);
		}.bind(this);

		this.loadDestinations();
	};

	Application.prototype.setSearchPage = function() {
		this.currentPage(PAGE_SEARCH);
	};

	Application.prototype.setRestorePage = function() {
		this.currentPage(PAGE_RESTORE);
	};

	Application.prototype.setStatusPage = function() {
		this.currentPage(PAGE_STATUS);
	};

	Application.prototype.setJobsPage = function() {
		this.currentPage(PAGE_JOBS);
	};

	Application.prototype.setSettingsPage = function() {
		this.currentPage(PAGE_SETTINGS);
	};

	Application.prototype.loadDestinations = function() {
		reqwest({
			url: "/api/v1/destination",
			type: "json",
			method: "POST",
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.destinations(resp.result);
					} else {
						console.error(resp.message);
					}
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					console.error(msg || err.responseText);
				}.bind(this)
			);
	};

	return Application;
});
