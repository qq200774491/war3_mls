-- 事件注册
-- 业务的eventname ，请不要用_开头
-- 事件参数用evalue, 最大长度1000个字节
-- room event api 
require('event/ms_event_api')
-- debug api
require('event/ms_event_debug')
-- ms_event_pong api
require('event/ms_event_pong')
-- ms_event_testapi api
require('event/ms_event_testapi')