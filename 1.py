import numpy as np
from scipy.interpolate import interp1d

# 构建插值函数
def build_interpolation_function(c_file):
    zhuanghaos = []
    gaochengs = []

    with open(c_file, 'r') as f:
        for line in f:
            parts = line.strip().split(',')
            if len(parts) == 2:
                zh = float(parts[0])
                gc = float(parts[1])
                zhuanghaos.append(zh)
                gaochengs.append(gc)

    # 转换为数组并排序
    zhuanghaos = np.array(zhuanghaos)
    gaochengs = np.array(gaochengs)
    sorted_indices = np.argsort(zhuanghaos)
    zhuanghaos = zhuanghaos[sorted_indices]
    gaochengs = gaochengs[sorted_indices]

    # 创建插值函数（线性）
    interpolator = interp1d(zhuanghaos, gaochengs, kind='linear', fill_value="extrapolate")

    return interpolator

# 主处理函数
def process_data(data_file, interpolator, output_file):
    result = []

    with open(data_file, 'r') as f:
        for line in f:
            parts = line.strip().split(',')
            if len(parts) == 3:
                zh = float(parts[0])       # 桩号
                data_gc = float(parts[1])  # data中的高程
                measure = parts[2]         # 测量值

                # 插值得到c.data对应高程
                try:
                    c_gc = interpolator(zh)
                except ValueError:
                    # 如果超出范围且没有设置 fill_value="extrapolate"，可以处理异常
                    c_gc = np.nan

                combined_gc = data_gc + c_gc

                # 添加结果
                result.append(f"{zh},{combined_gc},{measure}")

    # 写入文件
    with open(output_file, 'w') as f:
        f.write('\n'.join(result))

# 主程序入口
if __name__ == '__main__':
    c_file = 'c.data'
    data_file = 'data.txt'
    output_file = 'output.csv'

    # 构建插值函数
    interpolator = build_interpolation_function(c_file)

    # 处理数据并输出
    process_data(data_file, interpolator, output_file)

    print("处理完成，结果已写入 output.csv")