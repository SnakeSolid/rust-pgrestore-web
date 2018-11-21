"use strict";

define([ "knockout", "reqwest" ], function(ko, reqwest) {
	const BACKUP_PATH = "Path";
	const BACKUP_URL = "Url";
	const DATABASE_EXISTS = "Exists";
	const DATABASE_CREATE = "Create";
	const DATABASE_DROPANDCREATE = "DropAndCreate";
	const RESTORE_FULL = "Full";
	const RESTORE_SCHEMA = "Schema";
	const RESTORE_TABLES = "Tables";

	const EXTRACT_TABLES_RES = [
			/insert\s+into\s+(\w+\.\w+)\b/gi,
			/update\s+(\w+\.\w+)\b/gi,
			/from\s+(\w+\.\w+)\b/gi,
			/join\s+(\w+\.\w+)\b/gi,
		];
	const SEPARATORS_RE = /[\s,]+/;
	const WORDS_RE = /\w+/;

	const nonEmptyString = function(value) {
		return value.length > 0;
	};

	const Restore = function(params) {
		this.backup = params.backup;
		this.restoreCallback = params.restoreCallback;

		this.availableDestinations = ko.observableArray();
		this.selectedDestination = ko.observable();
		this.backupType = ko.observable(BACKUP_PATH);
		this.databaseName = ko.observable("");
		this.database = ko.observable(DATABASE_CREATE);
		this.restore = ko.observable(RESTORE_FULL);
		this.schemas = ko.observable("");
		this.tables = ko.observable("");
		this.ignoreErrors = ko.observable(false);
		this.parseSchemaVisible = ko.observable(false);
		this.parseTablesVisible = ko.observable(false);
		this.isLoading = ko.observable(false);
		this.isError = ko.observable(false);
		this.errorMessage = ko.observable();

		this.isBackupPath = ko.pureComputed(function() {
			return this.backupType() === BACKUP_PATH;
		}, this);

		this.isBackupUrl = ko.pureComputed(function() {
			return this.backupType() === BACKUP_URL;
		}, this);

		this.isDestinationInvalid = ko.pureComputed(function() {
			return this.selectedDestination() === undefined;
		}, this);

		this.isBackupPathInvalid = ko.pureComputed(function() {
			return this.backup().length === 0;
		}, this);

		this.isDatabaseNameInvalid = ko.pureComputed(function() {
			return this.databaseName().length === 0;
		}, this);

		this.isRestoreSchemasInvalid = ko.pureComputed(function() {
			return this.isRestoreSchemas() && !WORDS_RE.test(this.schemas());
		}, this);

		this.isRestoreTablesInvalid = ko.pureComputed(function() {
			return this.isRestoreTables() && !WORDS_RE.test(this.tables());
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

		this.isFormInvalid = ko.pureComputed(function() {
			return this.isDestinationInvalid()
				|| this.isBackupPathInvalid()
				|| this.isDatabaseNameInvalid()
				|| this.isRestoreSchemasInvalid()
				|| this.isRestoreTablesInvalid();
		}, this);

		this.schemaCallback = function(text) {
			this.restore(RESTORE_SCHEMA);
			this.schemas(this.parseSchema(text.toLowerCase()).sort().join(", "));
		}.bind(this);

		this.tablesCallback = function(text) {
			this.restore(RESTORE_TABLES);
			this.tables(this.parseTables(text.toLowerCase()).sort().join(", "));
		}.bind(this);

		this.backup.subscribe(function(value) {
			if (value.startsWith("http://") || value.startsWith("https://")) {
				this.backupType(BACKUP_URL);
			} else {
				this.backupType(BACKUP_PATH);
			}
		}.bind(this));

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
		const res = reqwest({
			url: "/api/v1/destination",
			type: "json",
  			method: "POST",
		}).then(function(resp) {
			if (resp.success) {
				this.availableDestinations(resp.result);
				this.isError(false);
			} else {
				this.isError(true);
				this.errorMessage(resp.message);
			}

			this.isLoading(false);
		}.bind(this)).fail(function(err, msg) {
			this.isLoading(false);
			this.isError(true);
			this.errorMessage(msg);
		}.bind(this));

		this.isLoading(true);
	};

	Restore.prototype.backupToCall = function() {
		const result = {};

		if (this.isBackupPath()) {
			result.type = BACKUP_PATH;
			result.path = this.backup();
		} else if (this.isBackupUrl()) {
			result.type = BACKUP_URL;
			result.url = this.backup();
		}

		return result;
	};

	Restore.prototype.restoreToCall = function() {
		const result = {};

		if (this.isRestoreFull()) {
			result.type = RESTORE_FULL;
		} else if (this.isRestoreSchemas()) {
			result.type = RESTORE_SCHEMA;
			result.schema = this.schemas().split(SEPARATORS_RE).filter(nonEmptyString);
		} else if (this.isRestoreTables()) {
			result.type = RESTORE_TABLES;
			result.tables = this.tables().split(SEPARATORS_RE).filter(nonEmptyString);
		}

		return result;
	};

	Restore.prototype.restoreDatabase = function() {
		const res = reqwest({
			url: "/api/v1/restore",
		  	type: "json",
  			method: "POST",
  			contentType: "application/json",
  			data: JSON.stringify({
				destination: this.selectedDestination(),
				backup: this.backupToCall(),
				database_name: this.databaseName(),
				database: this.database(),
				restore: this.restoreToCall(),
				ignore_errors: this.ignoreErrors(),
			}),
		}).then(function(resp) {
			if (resp.success) {
				this.restoreCallback(resp.result);
				this.isError(false);
			} else {
				this.isError(true);
				this.errorMessage(resp.message);
			}

			this.isLoading(false);
		}.bind(this)).fail(function(err, msg) {
			this.isLoading(false);
			this.isError(true);
			this.errorMessage(msg);
		}.bind(this));

		this.isLoading(true);
	};

	Restore.prototype.convertSlashes = function() {
		const backup = this.backup();
		const nForwardSlashes = backup.split(/\//).length;
		const nBackwardSlashes = backup.split(/\\/).length;

		if (nForwardSlashes > nBackwardSlashes) {
			this.backup(backup.replace(/\//g, "\\"));
		} else {
			this.backup(backup.replace(/\\/g, "/"));
		}
	};

	Restore.prototype.schemaFromCode = function() {
		this.parseSchemaVisible(true);
	};

	Restore.prototype.tablesFromCode = function() {
		this.parseTablesVisible(true);
	};

	Restore.prototype.parseTables = function(text) {
		const tables = new Set();

		for (const re of EXTRACT_TABLES_RES) {
			const result = text.match(re);

			if (result !== null) {
				result.map(function(item) {
					return re.exec(item)[1];
				}).forEach(function (item) {
					tables.add(item);
				});
			}
		}

		return Array.from(tables);
	};

	Restore.prototype.parseSchema = function(text) {
		const tables = this.parseTables(text);
		const schema = new Set();

		tables.forEach(function (tableName) {
			const index = tableName.indexOf(".");

			if (index > 0) {
				schema.add(tableName.substring(0, index));
			}
		});

		return Array.from(schema);
	};

	return Restore;
});
