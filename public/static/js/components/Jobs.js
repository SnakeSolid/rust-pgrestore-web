"use strict";

define(["knockout", "reqwest", "moment"], function(ko, reqwest, moment) {
	const DATE_FORMAT = "YYYY-MM-DD HH:mm:ss";

	const STATUS_PENDING = "Pending";
	const STATUS_IN_PROGRESS = "InProgress";
	const STATUS_SUCCESS = "Success";
	const STATUS_ABORTED = "Aborted";
	const STATUS_FAILED = "Failed";

	const compareJobs = function(a, b) {
		if (a.jobid < b.jobid) {
			return -1;
		} else if (a.jobid > b.jobid) {
			return 1;
		}

		return 0;
	};

	const Job = function(params) {
		this.jobid = params.jobid;
		this.started = moment.unix(params.created).format(DATE_FORMAT);
		this.updated = moment.unix(params.modified).format(DATE_FORMAT);
		this.databaseName = params.database_name;
		this.status = params.status;
		this.stage = params.stage;

		this.isSuccess = ko.pureComputed(function() {
			return this.status === STATUS_SUCCESS;
		}, this);

		this.isAborted = ko.pureComputed(function() {
			return this.status === STATUS_ABORTED;
		}, this);

		this.isFailed = ko.pureComputed(function() {
			return this.status === STATUS_FAILED;
		}, this);

		this.statusString = ko.pureComputed(
			function() {
				switch (this.status) {
					case STATUS_PENDING:
						return "Pending";
					case STATUS_IN_PROGRESS:
						return "In progress";
					case STATUS_SUCCESS:
						return "Finished with success";
					case STATUS_ABORTED:
						return "Aborted";
					case STATUS_FAILED:
						return "Failed";
				}
			}.bind(this)
		);
	};

	const Jobs = function(params) {
		this.callback = params.callback;

		this.isLoading = ko.observable(false);
		this.isError = ko.observable(false);
		this.errorMessage = ko.observable("");
		this.results = ko.observableArray([]);

		this.hasResults = ko.pureComputed(function() {
			return this.results().length > 0;
		}, this);

		this.selectJob = function(job) {
			this.callback(job.jobid);
		}.bind(this);
	};

	Jobs.prototype.updateJobs = function() {
		const self = this;
		const res = reqwest({
			url: "/api/v2/jobs",
			type: "json",
			method: "POST",
		})
			.then(
				function(resp) {
					if (resp.success) {
						const data = resp.result.sort(compareJobs).map(function(params) {
							return new Job(params);
						});

						this.results(data);
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

		this.isLoading(false);
		this.isError(false);
		this.errorMessage("");
	};

	return Jobs;
});
