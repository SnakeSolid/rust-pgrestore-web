"use strict";

define([ "knockout" ], function(ko) {
	const PAGE_SEARCH = "Search";
	const PAGE_RESTORE = "Restore";
	const PAGE_STATUS = "Status";
	const PAGE_JOBS = "Jobs";

	const Application = function() {
		this.currentPage = ko.observable(PAGE_RESTORE);
		this.currentJobid = ko.observable();
		this.backupSearchResult = ko.observable("");

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

		this.searchCallback = function(value) {
			this.backupSearchResult(value);
			this.currentPage(PAGE_RESTORE);
		}.bind(this);

		this.restoreCallback = function(jobid) {
			this.currentJobid(jobid);
			this.currentPage(PAGE_STATUS);
		}.bind(this);
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

	return Application;
});
