import json
import sys

url = str(sys.argv[1])
db = str(sys.argv[2])
username = str(sys.argv[3])
password = str(sys.argv[4])
skills = json.loads(str(sys.argv[5]))

import xmlrpc.client

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))

uid = common.authenticate(db, username, password, {})

models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))
odoo_skills = models.execute_kw(db, uid, password,
    'x_hula_skill', 'search_read', [], {'fields': ['id', 'display_name']})
lowercased_label_to_odoo_skill = {s['display_name'].lower(): s for s in odoo_skills}

created_skills = []
for skill in skills:
    label = skill['label']
    lowercased_label = label.lower()

    found_odoo_skill = lowercased_label_to_odoo_skill.get(lowercased_label, None)
    if not found_odoo_skill:
        models.execute_kw(db, uid, password, 'x_hula_skill', 'create', [{
            'x_name': label,
            'display_name': label
        }])
        created_skills.append(label)

print(json.dumps(created_skills))
