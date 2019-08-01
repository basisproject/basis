const Exonum = require('exonum-client');
const rp = require('request-promise');
const protobuf = require('./protobuf');
const config = require('./config');

const types = {};
const message_id_map = {
	'user.TxCreate': 0,
	'user.TxUpdate': 1,
	'user.TxSetPubkey': 2,
	'user.TxSetRoles': 3,
	'user.TxDelete': 4,

    'company.TxCreatePrivate': 5,
    'company.TxUpdate': 6,
    'company.TxSetType': 7,
    'company.TxDelete': 8,

    'company_member.TxCreate': 9,
    'company_member.TxSetRoles': 10,
    'company_member.TxDelete': 11,
};
Object.keys(message_id_map).forEach((key) => {
	const [type, tx] = key.split('.');
	if(!types[type]) types[type] = {};
	types[type][tx] = {
		type: `factor.${key}`,
		msg_id: message_id_map[key],
	}
});

exports.types = types;

exports.make = (type, data, params) => {
	const Transaction = protobuf.root.lookupType(type.type);
	const map_types = {
		'google.protobuf.Timestamp': 'Timestamp',
		'exonum.Hash': 'Hash',
		'exonum.PublicKey': 'Pubkey',
		'CompanyType': 'CompanyType',
	};
	Object.keys(Transaction.fields).forEach((field) => {
		if(typeof(data[field]) == 'undefined') return;
		const spec = Transaction.fields[field];
		const mapping = map_types[spec.type];
		if(!mapping) return;
		data[field] = protobuf.types[mapping].gen(data[field]);
	});
	const errmsg = Transaction.verify(data);
	if(errmsg) {
		throw new Error(`transaction: ${type}: verification error: ${errmsg}`);
	}
	const trans = Exonum.newTransaction({
		schema: Transaction,
		author: params.pubkey,
		service_id: params.service_id || config.service_id,
		message_id: params.message_id || type.msg_id,
	});
	return trans;
};

exports.send = async (type, data, params) => {
	const trans = exports.make(type, data, params);
	return trans.send(`${config.endpoint}/explorer/v1/transactions`, data, params.privkey);
};

let whoswho = {};
exports.add_user = (who, pubkey, seckey) => {
	whoswho[who] = {pub: pubkey, sec: seckey};
};

exports.clear_users = () => {
	whoswho = {};
};

exports.send_as = async (who, type, data, params) => {
	const user = whoswho[who];
	if(!user) throw new Error(`helpers/transactions::send_as() -- missing user ${who}`);
	const newparams = Object.assign({}, {
		pubkey: user.pub,
		privkey: user.sec,
	}, params || {});
	return exports.send(type, data, newparams);
};

exports.get = async (txid) => {
	const res = await rp({
		url: `${config.endpoint}/explorer/v1/transactions?hash=${txid}`,
		json: true,
	});
	return res;
};

exports.status = async (txid) => {
	const res = await exports.get(txid);
	if(!res || res.type == 'unknown') return {missing: true};
	return {
		committed: res.type == 'committed',
		success: res.status.type == 'success',
	};
};
