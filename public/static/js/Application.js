"use strict";

define([ "knockout", "moment", "reqwest" ], function(ko, moment, reqwest) {
	const DATABASE_EXISTS = "Exists";
	const DATABASE_CREATE = "Create";
	const DATABASE_DROPANDCREATE = "DropAndCreate";
	const RESTORE_FULL = "Full";
	const RESTORE_SCHEMA = "Schema";
	const RESTORE_TABLES = "Tables";

	const Application = function() {
		this.loading = ko.observable(false);
		this.error = ko.observable(false);
		this.errorMessage = ko.observable();
		this.availableDestinations = ko.observableArray();
		this.selectedDestination = ko.observable();
		this.backupPath = ko.observable("");
		this.databaseName = ko.observable("");
		this.database = ko.observable(DATABASE_CREATE);
		this.restore = ko.observable(RESTORE_FULL);
		this.schemas = ko.observable("");
		this.tables = ko.observable("");

		this.isLoading = ko.pureComputed(function() {
			return this.loading();
		}, this);

		this.isError = ko.pureComputed(function() {
			return this.error();
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
	};

	Application.prototype.setDatabaseExists = function() {
		this.database(DATABASE_EXISTS);
	};

	Application.prototype.setDatabaseCreate = function() {
		this.database(DATABASE_CREATE);
	};

	Application.prototype.setDatabaseDropAndCreate = function() {
		this.database(DATABASE_DROPANDCREATE);
	};

	Application.prototype.setRestoreFull = function() {
		this.restore(RESTORE_FULL);
	};

	Application.prototype.setRestoreSchemas = function() {
		this.restore(RESTORE_SCHEMA);
	};

	Application.prototype.setRestoreTables = function() {
		this.restore(RESTORE_TABLES);
	};

	Application.prototype.loadDestinations = function() {
		const self = this;
		const res = reqwest({
			url: "/api/v1/destination",
			type: "json",
  			method: "POST",
		}).then(function(resp) {
			if (resp.success) {
				self.availableDestinations(resp.result);
				self.error(false);
			} else {
				self.error(true);
				self.errorMessage(resp.message);
			}

			self.loading(false);
		}).fail(function(err, msg) {
			self.loading(false);
			self.error(true);
			self.errorMessage(msg);
		});

		this.loading(true);
	};

	Application.prototype.restoreDatabase = function() {
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
				alert(JSON.stringify(resp));
				self.error(false);
			} else {
				self.error(true);
				self.errorMessage(resp.message);
			}

			self.loading(false);
		}).fail(function(err, msg) {
			self.loading(false);
			self.error(true);
			self.errorMessage(msg);
		});

		this.loading(true);
	};

	Application.prototype.convertSlashes = function() {
		const backupPath = this.backupPath();
		const nForwardSlashes = backupPath.split(/\//).length;
		const nBackwardSlashes = backupPath.split(/\\/).length;

		if (nForwardSlashes > nBackwardSlashes) {
			this.backupPath(backupPath.replace(/\//g, "\\"));
		} else {
			this.backupPath(backupPath.replace(/\\/g, "/"));
		}
	};

	return Application;
});
