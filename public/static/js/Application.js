"use strict";

define([ "knockout" ], function(ko) {
	const PAGE_RESTORE = "Restore";
	const PAGE_STATUS = "Status";
	const PAGE_JOBS = "Jobs";

	const Application = function() {
		this.currentPage = ko.observable(PAGE_RESTORE);
		this.currentJobid = ko.observable();

		this.isRestoreVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_RESTORE;
		}, this);

		this.isStatusVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_STATUS;
		}, this);

		this.isJobsVisible = ko.pureComputed(function() {
			return this.currentPage() === PAGE_JOBS;
		}, this);

		const self = this;

		// TODO Replace this workaround with correct solution
		// this bind caller context instead of application
		this.restoreCallback = function(jobid) {
			self.currentJobid(jobid);
			self.currentPage(PAGE_STATUS);
		};
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
