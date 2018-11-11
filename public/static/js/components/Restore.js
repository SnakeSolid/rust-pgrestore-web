"use strict";

define([ "knockout", "reqwest" ], function(ko, reqwest) {
	const DATABASE_EXISTS = "Exists";
	const DATABASE_CREATE = "Create";
	const DATABASE_DROPANDCREATE = "DropAndCreate";
	const RESTORE_FULL = "Full";
	const RESTORE_SCHEMA = "Schema";
	const RESTORE_TABLES = "Tables";

	const Restore = function(params) {
		this.restoreCallback = params.restoreCallback;

		this.availableDestinations = ko.observableArray();
		this.selectedDestination = ko.observable();
		this.backupPath = ko.observable("");
		this.databaseName = ko.observable("");
		this.database = ko.observable(DATABASE_CREATE);
		this.restore = ko.observable(RESTORE_FULL);
		this.schemas = ko.observable("");
		this.tables = ko.observable("");
		this.isLoading = ko.observable(false);
		this.isError = ko.observable(false);
		this.errorMessage = ko.observable();

		this.isDestinationInvalid = ko.pureComputed(function() {
			return this.selectedDestination() === undefined;
		}, this);

		this.isBackupPathInvalid = ko.pureComputed(function() {
			return this.backupPath().length === 0;
		}, this);

		this.isDatabaseNameInvalid = ko.pureComputed(function() {
			return this.databaseName().length === 0;
		}, this);

		this.isRestoreFull = ko.pureComputed(function() {
			return this.restore() === RESTORE_FULL;
		}, this);

		this.isRestoreSchemas = ko.pureComputed(function() {
			return this.restore() === RESTORE_SCHEMA;
		}, this);

		this.isRestoreTables = ko.pureComputed(function() {
			return this.restore() === RESTORE_TABLES;
		}, this);

		this.loadDestinations();
	};

	Restore.prototype.setDatabaseExists = function() {
		this.database(DATABASE_EXISTS);
	};

	Restore.prototype.setDatabaseCreate = function() {
		this.database(DATABASE_CREATE);
	};

	Restore.prototype.setDatabaseDropAndCreate = function() {
		this.database(DATABASE_DROPANDCREATE);
	};

	Restore.prototype.setRestoreFull = function() {
		this.restore(RESTORE_FULL);
	};

	Restore.prototype.setRestoreSchemas = function() {
		this.restore(RESTORE_SCHEMA);
	};

	Restore.prototype.setRestoreTables = function() {
		this.restore(RESTORE_TABLES);
	};

	Restore.prototype.loadDestinations = function() {
		const self = this;
		const res = reqwest({
			url: "/api/v1/destination",
			type: "json",
  			method: "POST",
		}).then(function(resp) {
			if (resp.success) {
				self.availableDestinations(resp.result);
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

	Restore.prototype.restoreDatabase = function() {
		const self = this;
		const res = reqwest({
			url: "/api/v1/restore",
		  	type: "json",
  			method: "POST",
  			contentType: "pplication/json",
  			data: JSON.stringify({
  			  				destination: self.selectedDestination(),
  			  				backup_path: self.backupPath(),
  			  				database_name: self.databaseName(),
  			  				database: self.database(),
  			  				restore: self.restore(),
  			  			}),
		}).then(function(resp) {
			if (resp.success) {
				self.restoreCallback(resp.result);
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

	Restore.prototype.convertSlashes = function() {
		const backupPath = this.backupPath();
		const nForwardSlashes = backupPath.split(/\//).length;
		const nBackwardSlashes = backupPath.split(/\\/).length;

		if (nForwardSlashes > nBackwardSlashes) {
			this.backupPath(backupPath.replace(/\//g, "\\"));
		} else {
			this.backupPath(backupPath.replace(/\\/g, "/"));
		}
	};

	return Restore;
});
