"use strict";

define([ "knockout", "reqwest" ], function(ko, reqwest) {
	const STATUS_INPROGRESS = "InProgress";
	const STATUS_SUCCESS = "Success";
	const STATUS_FAILED = "Failed";

	const Status = function(params) {
		this.jobid = params.jobid;
		this.currectTimer = null;

		this.stage = ko.observable("");
		this.stdout = ko.observable("");
		this.stderr = ko.observable("");
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

	Status.prototype.checkJobid = function(newValue) {
		if (newValue !== undefined) {
			this.startTimer();
		}

		this.stage("");
		this.stdout("");
		this.stderr("");
		this.status(STATUS_INPROGRESS);
	};

	Status.prototype.updateStatus = function() {
		const self = this;
		const res = reqwest({
			url: "/api/v1/status",
			type: "json",
  			method: "POST",
  			contentType: "pplication/json",
  			data: JSON.stringify({ jobid: self.jobid(), }),
		}).then(function(resp) {
			if (resp.success) {
				const data = resp.result;

				self.stage(data.stage);
				self.stdout(data.stdout);
				self.stderr(data.stderr);
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
