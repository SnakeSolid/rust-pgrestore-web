"use strict";

define(["knockout", "reqwest"], function(ko, reqwest) {
	const STATUS_LOADING = "Loading";
	const STATUS_INPROGRESS = "InProgress";
	const STATUS_SUCCESS = "Success";
	const STATUS_ABORTED = "Aborted";
	const STATUS_FAILED = "Failed";

	const MAX_OUTPUT_LENGTH = 8192;

	const Status = function(params) {
		this.jobid = params.jobid;
		this.timerId = undefined;
		this.stdoutPosition = 0;
		this.stderrPosition = 0;

		this.databaseName = ko.observable("");
		this.stage = ko.observable("");
		this.stdout = ko.observable("");
		this.stderr = ko.observable("");
		this.stdoutTrimmed = ko.observable(false);
		this.stderrTrimmed = ko.observable(false);
		this.status = ko.observable(STATUS_INPROGRESS);
		this.isAbortDisabled = ko.observable(false);
		this.truncateOutput = ko.observable(true);

		this.isJobDefined = ko.pureComputed(function() {
			return this.jobid() !== undefined;
		}, this);

		this.isJobUndefined = ko.pureComputed(function() {
			return this.jobid() === undefined;
		}, this);

		this.isLoading = ko.pureComputed(function() {
			return this.status() === STATUS_LOADING;
		}, this);

		this.isInProgress = ko.pureComputed(function() {
			return this.status() === STATUS_INPROGRESS;
		}, this);

		this.isSuccess = ko.pureComputed(function() {
			return this.status() === STATUS_SUCCESS;
		}, this);

		this.isAborted = ko.pureComputed(function() {
			return this.status() === STATUS_ABORTED;
		}, this);

		this.isFailed = ko.pureComputed(function() {
			return this.status() === STATUS_FAILED;
		}, this);

		this.hasStdout = ko.pureComputed(function() {
			return this.stdout().length > 0;
		}, this);

		this.hasStderr = ko.pureComputed(function() {
			return this.stderr().length > 0;
		}, this);

		this.isStdoutTrimmed = ko.pureComputed(function() {
			return this.stdout().length > 0 && this.stdoutTrimmed();
		}, this);

		this.isStderrTrimmed = ko.pureComputed(function() {
			return this.stderr().length > 0 && this.stderrTrimmed();
		}, this);

		this.checkJobid(this.jobid());
		this.jobid.subscribe(this.checkJobid, this);
	};

	Status.prototype.reset = function() {
		this.stdoutPosition = 0;
		this.stderrPosition = 0;

		this.databaseName("");
		this.stage("");
		this.stdout("");
		this.stderr("");
		this.stdoutTrimmed(false);
		this.stderrTrimmed(false);
		this.isAbortDisabled(false);
		this.status(STATUS_LOADING);
	};

	Status.prototype.checkJobid = function(newValue) {
		this.reset();

		if (newValue !== undefined) {
			if (this.timerId !== undefined) {
				clearTimeout(this.timerId);
			}

			this.updateStatus();
		}
	};

	Status.prototype.trimValue = function(value, flag) {
		if (this.truncateOutput() && value.length > MAX_OUTPUT_LENGTH) {
			flag(true);

			return value.substring(value.length - MAX_OUTPUT_LENGTH);
		} else {
			return value;
		}
	};

	Status.prototype.abortJob = function() {
		reqwest({
			url: "/api/v3/abort",
			type: "json",
			method: "POST",
			contentType: "application/json",
			data: JSON.stringify({
				jobid: this.jobid(),
			}),
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.isAbortDisabled(true);
					} else {
						this.isAbortDisabled(false);
					}
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.isAbortDisabled(false);
				}.bind(this)
			);
	};

	Status.prototype.updateStatus = function() {
		reqwest({
			url: "/api/v3/status",
			type: "json",
			method: "POST",
			contentType: "application/json",
			data: JSON.stringify({
				jobid: this.jobid(),
				stdout_position: this.stdoutPosition,
				stderr_position: this.stderrPosition,
			}),
		})
			.then(
				function(resp) {
					if (resp.success) {
						const data = resp.result;

						this.stdoutPosition = data.stdout_position;
						this.stderrPosition = data.stderr_position;

						this.databaseName(data.database_name);
						this.stage(data.stage);
						this.stdout(this.trimValue(this.stdout() + data.stdout, this.stdoutTrimmed));
						this.stderr(this.trimValue(this.stderr() + data.stderr, this.stderrTrimmed));
						this.status(data.status);

						if (data.status === STATUS_INPROGRESS) {
							this.timerId = setTimeout(this.updateStatus.bind(this), 1000);
						} else {
							this.timerId = undefined;
						}
					}
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.timerId = undefined;
				}.bind(this)
			);
	};

	return Status;
});
