"use strict";

define([ "knockout", "reqwest" ], function(ko, reqwest) {
	const STATUS_INPROGRESS = "InProgress";
	const STATUS_SUCCESS = "Success";
	const STATUS_FAILED = "Failed";

	const MAX_OUTPUT_LENGTH = 8192;

	const Status = function(params) {
		this.jobid = params.jobid;
		this.stdoutPosition = 0;
		this.stderrPosition = 0;

		this.stage = ko.observable("");
		this.stdout = ko.observable("");
		this.stderr = ko.observable("");
		this.stdoutTrimmed = ko.observable(false);
		this.stderrTrimmed = ko.observable(false);
		this.status = ko.observable(STATUS_INPROGRESS);

		this.isJobDefined = ko.pureComputed(function() {
			return this.jobid() !== undefined;
		}, this);

		this.isJobUndefined = ko.pureComputed(function() {
			return this.jobid() === undefined;
		}, this);

		this.isInProgress = ko.pureComputed(function() {
			return this.status() === STATUS_INPROGRESS;
		}, this);

		this.isSuccess = ko.pureComputed(function() {
			return this.status() === STATUS_SUCCESS;
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

		this.stage("");
		this.stdout("");
		this.stderr("");
		this.stdoutTrimmed(false);
		this.stderrTrimmed(false);
		this.status(STATUS_INPROGRESS);
	};

	Status.prototype.checkJobid = function(newValue) {
		this.reset();

		if (newValue !== undefined) {
			this.updateStatus();
		}
	};

	Status.prototype.trimValue = function(value, flag) {
		if (value.length > MAX_OUTPUT_LENGTH) {
			flag(true);

			return value.substring(value.length - MAX_OUTPUT_LENGTH);
		} else {
			return value;
		}
	}

	Status.prototype.updateStatus = function() {
		const res = reqwest({
			url: "/api/v1/status",
			type: "json",
			method: "POST",
			contentType: "application/json",
			data: JSON.stringify({
				jobid: this.jobid(),
				stdout_position: this.stdoutPosition,
				stderr_position: this.stderrPosition,
			}),
		}).then(function(resp) {
			if (resp.success) {
				const data = resp.result;

				this.stdoutPosition = data.stdout_position;
				this.stderrPosition = data.stderr_position;

				this.stage(data.stage);
				this.stdout(this.trimValue(this.stdout() + data.stdout, this.stdoutTrimmed));
				this.stderr(this.trimValue(this.stderr() + data.stderr, this.stderrTrimmed));
				this.status(data.status);

				if (data.status === STATUS_INPROGRESS) {
					setTimeout(this.updateStatus.bind(this), 1000);
				}
			}
		}.bind(this));
	};

	return Status;
});
