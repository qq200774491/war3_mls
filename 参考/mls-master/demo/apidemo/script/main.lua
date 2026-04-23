-- 初始化随机数种子
math.randomseed(os.time())

-- 加载基础API
require('com_api')

-- 加载加载的测试接口
require('ms_api')

-- 游戏入口
require('game_entry')
