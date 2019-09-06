"use strict";

const fs = require('fs');
const path = require('path');
const Promise = require('bluebird');
const protobuf = require('protobufjs');
const Exonum = require('exonum-client');
const config = require('./config');

const Timestamp = {
	type: (function() {
		const Type = new protobuf.Type('Timestamp');
		Type.add(new protobuf.Field('seconds', 1, 'int64'));
		Type.add(new protobuf.Field('nanos', 2, 'int32'));
		return Type;
	})(),
	gen: function(datestr) {
		const now = new Date(datestr).getTime();
		const seconds = Math.floor(now / 1000);
		return {
			seconds: seconds,
			nanos: (now - (seconds * 1000)) * 1000000,
		};
	},
	from: function(date) {
		let ts = parseInt((date.seconds * 1000) + (date.nanos / 1000000));
		return new Date(ts);
	},
};

const Hash = {
	type: (function() {
		const Type = new protobuf.Type('Hash');
		Type.add(new protobuf.Field('data', 1, 'bytes'));
		return Type;
	})(),
	gen: function(pubkey) {
		return {data: Exonum.hexadecimalToUint8Array(pubkey)};
	},
};

const Pubkey = {
	type: (function() {
		const Type = new protobuf.Type('PublicKey');
		Type.add(new protobuf.Field('data', 1, 'bytes'));
		return Type;
	})(),
	gen: function(pubkey) {
		return {data: Exonum.hexadecimalToUint8Array(pubkey)};
	},
};

const CompanyType = {
	map: {
		UNKNOWN: 0,
		PUBLIC: 1,
		SYNDICATE: 2,
		PRIVATE: 3,
	},
	type: new protobuf.Enum('CompanyType', this.map),
	gen: function(val) {
		return CompanyType.map[val.toUpperCase()] || 0;
	},
};

const Unit = {
	map: {
		UNKNOWN: 0,
		MILLIMETER: 1,
		MILLILITER: 2,
		WATTHOUR: 3,
	},
	type: new protobuf.Enum('Unit', this.map),
	gen: function(val) {
		return Unit.map[val.toUpperCase()] || 0;
	},
};

const Time = {
	map: {
		UNKNOWN: 0,
		NANOSECONDS: 1,
		MILLISECONDS: 2,
		SECONDS: 3,
		MINUTES: 4,
		HOURS: 5,
		DAYS: 6,
		WEEKS: 7,
		YEARS: 8,
	},
	type: new protobuf.Enum('Time', this.map),
	gen: function(val) {
		return Time.map[val.toUpperCase()] || 0;
	},
};

// NOTE: this must usually also be mapped in helpers/transactions::make()
exports.types = {
	Timestamp: Timestamp,
	Hash: Hash,
	Pubkey: Pubkey,
	CompanyType: CompanyType,
	Unit: Unit,
	Time: Time,
};

const protos = new protobuf.Root();
protos.resolvePath = (origin, target) => {
	return target;
};

protos.define('exonum').add(Pubkey.type);
protos.define('exonum').add(Hash.type);
exports.root = protos;

function load() {
	const files = fs.readdirSync(config.protobuf_dir)
	files.forEach(function(protofile) {
		if(protofile.match(/^\./) || !protofile.match(/\.proto$/)) return;
		const fullpath = fs.realpathSync(config.protobuf_dir+'/'+protofile);
		const name = path.basename(fullpath, '.proto');
		if(protos[name]) return protos[name];
		protos.loadSync(fullpath, {keepCase: true});
	});
};
load();

