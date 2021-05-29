import json

url1 = 'https://giraffe-code2.odoo.com'
db1 = 'giraffe-code2'
username1 = 'sintopek@gmail.com'
password1 = '1eea396dd60a79e9f5f9ef43e74aeade613d1b57'

import xmlrpc.client
#info = xmlrpc.client.ServerProxy(url1).start()
#url, db, username, password = \
#    info['host'], info['database'], info['user'], info['password']

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url1))
v = common.version()

print(v)
print("----")


uid = common.authenticate(db1, username1, password1, {})

print(uid)

models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url1))
a = models.execute_kw(db1, uid, password1,
    'res.partner', 'check_access_rights',
    ['read'], {'raise_exception': False})

b = models.execute_kw(db1, uid, password1,
    'crm.lead', 'check_access_rights',
    ['read'], {'raise_exception': False})


c = models.execute_kw(db1, uid, password1,
    'crm.lead', 'search_read',
    [[]],
    {'fields': ['name'], 'limit': 5})



print(c)

d = models.execute_kw(
    db1, uid, password1, 'hr.employee', 'fields_get',
    [], {'attributes': ['string', 'help', 'type']})


#print(json.dumps(d))



