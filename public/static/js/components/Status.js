"use strict";

define([ "knockout", "reqwest" ], function(ko, reqwest) {
	const STATUS_INPROGRESS = "InProgress";
	const STATUS_SUCCESS = "Success";
	const STATUS_FAILED = "Failed";

	const MAX_OUTPUT_LENGTH = 8192;

	const Status = function(params) {
		this.jobid = params.jobid;
		this.currectTimer = null;
		this.stdoutPosition = 0;
		this.stderrPosition = 0;

		this.stage = ko.observable("");
		this.stdout = ko.observable("");
		this.stderr = ko.observable("");
		this.stdoutTrimmed = ko.observable(false);
		this.stderrTrimmed = ko.observable(false);
		this.status = ko.observable(STATUS_INPROGRESS);

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

	Status.prototype.startTimer = function() {
		this.stopTimer();
		this.updateStatus()
		this.currectTimer = setInterval(this.updateStatus.bind(this), 1000);
	};

	Status.prototype.stopTimer = function() {
		if (this.currectTimer != null) {
			clearInterval(this.currectTimer);

			this.currectTimer = null;
		}
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
			this.startTimer();
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
		const self = this;
		const res = reqwest({
			url: "/api/v1/status",
			type: "json",
			method: "POST",
			contentType: "application/json",
			data: JSON.stringify({
				jobid: self.jobid(),
				stdout_position: self.stdoutPosition,
				stderr_position: self.stderrPosition,
			}),
		}).then(function(resp) {
			if (resp.success) {
				const data = resp.result;

				self.stdoutPosition = data.stdout_position;
				self.stderrPosition = data.stderr_position;

				self.stage(data.stage);
				self.stdout(self.trimValue(self.stdout() + data.stdout, self.stdoutTrimmed));
				self.stderr(self.trimValue(self.stderr() + data.stderr, self.stderrTrimmed));
				self.status(data.status);

				if (data.status === STATUS_SUCCESS || data.status === STATUS_FAILED) {
					self.stopTimer();
				}
			} else {
				self.stopTimer();
			}
		}).fail(function(err, msg) {
			self.stopTimer();
		});
	};

	return Status;
});
