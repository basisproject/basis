const Promise = require('bluebird');
const config = require('../helpers/config');
const transactions = require('../helpers/transactions');
const Users = require('../helpers/users');

exports.load = async function() {
	const data = {
		id: config.bootstrap_user.id,
		pubkey: config.bootstrap_user.pub,
		roles: ['SuperAdmin'],
		email: config.bootstrap_user.email,
		name: config.bootstrap_user.name,
		meta: '{}',
		created: new Date().toISOString(),
	};
	const params = {
		pubkey: config.bootstrap_user.pub,
		privkey: config.bootstrap_user.sec,
		message_id: 0,
	};
	const txid = await transactions.send('factor.user.TxCreate', data, params);
	await Promise.delay(200);
	const status = await transactions.status(txid);
	if(!status.success) {
		throw new Error('helpers/bootstrap::load() -- user create failed: '+JSON.stringify(status));
	}
	return txid;
};

exports.unload = async function() {
	const data = {
		id: config.bootstrap_user.id,
		memo: 'You are *fired* =D!!',
		deleted: new Date().toISOString(),
	};
	const params = {
		pubkey: config.bootstrap_user.pub,
		privkey: config.bootstrap_user.sec,
		message_id: 4,
	};
	const txid = await transactions.send('factor.user.TxDelete', data, params);
	await Promise.delay(100);
	const status = await transactions.status(txid);
	if(!status.success) {
		throw new Error('helpers/bootstrap::unload() -- user delete failed: '+JSON.stringify(status));
	}
};

