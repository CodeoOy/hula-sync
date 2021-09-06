import json
import sys
import jsonpickle
from datetime import timedelta
from datetime import datetime

url = str(sys.argv[1])
db = str(sys.argv[2])
username = str(sys.argv[3])
password = str(sys.argv[4])
lastrun = str(sys.argv[5])

import xmlrpc.client

class Project(object):
        id :str = ""
        name :str = ""
        description :str = ""
        visible :bool = False
        needs = []

class ProjectNeed(object):
        label :str = ""
        nbr :int = ""
        begin :datetime
        end :datetime
        skills = []

class ProjectNeedSkill(object):
        skill :str = ""
        level :str = ""
        min_years :int
        mandatory :bool

if str(lastrun):
        t = datetime.utcnow() - timedelta(minutes=int(lastrun))
        limit = t.strftime('%Y-%m-%d %H:%M:%S')
else:
        t = datetime.min
        limit = t.strftime('%Y-%m-%d %H:%M:%S')

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))

uid = common.authenticate(db, username, password, {})

models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))
c = models.execute_kw(db, uid, password,
    'crm.lead', 'search_read',
    [[['write_date', '>', limit]]],
    {'fields': ['id', 
        'name',
        'x_studio_project_name', 
        'x_studio_description', 
        'x_studio_begin', 
        'x_studio_end', 
        'x_studio_nbr_of_positions', 
        'x_studio_skill_1', 
        'x_studio_level_1', 
        'x_studio_minimum_years_1', 
        'x_studio_mandatory_1', 
        'x_studio_skill_2', 
        'x_studio_level_2', 
        'x_studio_minimum_years_2', 
        'x_studio_mandatory_2',
        'x_studio_skill_3', 
        'x_studio_level_3', 
        'x_studio_minimum_years_3', 
        'x_studio_mandatory_3', 
        'x_studio_skill_4', 
        'x_studio_level_4', 
        'x_studio_minimum_years_4', 
        'x_studio_mandatory_4',
        'x_studio_role',
        'x_studio_visible'],
        'limit': 5000})

res = []

for cc in c:
    if not cc['x_studio_skill_1']:
            continue

    if isinstance(cc['x_studio_begin'], bool):
            continue

    if not cc['x_studio_project_name']:
            cc['x_studio_project_name'] = 'Odoo project: ' + str(cc['id'])

    if not cc['x_studio_description']:
            cc['x_studio_description'] = cc['x_studio_project_name']

    if not cc['x_studio_role']:
            cc['x_studio_role'] = cc['x_studio_project_name']

    if isinstance(cc['x_studio_level_1'], bool):
            cc['x_studio_level_1'] = None
    else:
            cc['x_studio_level_1'] = cc['x_studio_level_1'][1]                        

    if isinstance(cc['x_studio_level_2'], bool):
            cc['x_studio_level_2'] = None
    else:
            cc['x_studio_level_2'] = cc['x_studio_level_2'][1]                        

    if isinstance(cc['x_studio_level_3'], bool):
            cc['x_studio_level_3'] = None
    else:
            cc['x_studio_level_3'] = cc['x_studio_level_3'][1]                        

    if isinstance(cc['x_studio_level_4'], bool):
            cc['x_studio_level_4'] = None
    else:
            cc['x_studio_level_4'] = cc['x_studio_level_4'][1]                        

    if isinstance(cc['x_studio_end'], bool):
            cc['x_studio_end'] = None


    skills = []
    if cc['x_studio_skill_1']:
        skill = ProjectNeedSkill()
        skill.skill = cc['x_studio_skill_1'][1]
        skill.level = cc['x_studio_level_1']
        skill.min_years = cc['x_studio_minimum_years_1'] 
        skill.mandatory = cc['x_studio_mandatory_1']
        skills.append(skill)

    if cc['x_studio_skill_2']:
        skill2 = ProjectNeedSkill()
        skill2.skill = cc['x_studio_skill_2'][1]
        skill2.level = cc['x_studio_level_2']
        skill2.min_years = cc['x_studio_minimum_years_2'] 
        skill2.mandatory = cc['x_studio_mandatory_2']
        skills.append(skill2)

    if cc['x_studio_skill_3']:
        skill3 = ProjectNeedSkill()
        skill3.skill = cc['x_studio_skill_3'][1]
        skill3.level = cc['x_studio_level_3']
        skill3.min_years = cc['x_studio_minimum_years_3'] 
        skill3.mandatory = cc['x_studio_mandatory_3']
        skills.append(skill3)

    if cc['x_studio_skill_4']:
        skill4 = ProjectNeedSkill()
        skill4.skill = cc['x_studio_skill_4'][1]
        skill4.level = cc['x_studio_level_4']
        skill4.min_years = cc['x_studio_minimum_years_4'] 
        skill4.mandatory = cc['x_studio_mandatory_4']
        skills.append(skill4)

    needs = []
    need = ProjectNeed()
    need.label = cc['x_studio_role']
    need.nbr = cc['x_studio_nbr_of_positions']
    need.begin = cc['x_studio_begin']
    need.end = cc['x_studio_end']
    need.skills = skills

    proj = Project()
    proj.id = cc['id']
    proj.name = cc['x_studio_project_name']
    proj.description = cc['x_studio_description']
    proj.visible = cc['x_studio_visible']
    proj.needs = [need]

    res.append(proj)

print(jsonpickle.encode(res, unpicklable=False))

#for r in res:
#    print(jsonpickle.encode(r, unpicklable=False))

