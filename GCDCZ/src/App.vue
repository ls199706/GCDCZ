<script setup>
import { ref, watch } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

// 文件路径
const cFilePath = ref("");
const dataFilePath = ref("");

// 预览数据 - 高程数据
const previewData = ref([]);
const isLoadingPreview = ref(false);
const previewError = ref("");

// 预览数据 - 待处理数据
const dataPreviewData = ref([]);
const isLoadingDataPreview = ref(false);
const dataPreviewError = ref("");

// 状态变量
const isProcessing = ref(false);
const processMessage = ref('');

// 辅助函数：检查字符串是否为数字
const isNumeric = (str) => !isNaN(str) && !isNaN(parseFloat(str));

// 辅助函数：检查一行文本是否包含数字
const isLineContainsNumbers = (line) => /\d/.test(line);

// 辅助函数：智能解析CSV行，处理引号包围的值
const parseCsvLine = (line) => {
  // 简单的CSV解析器，处理引号包围的值
  const parts = [];
  let current = '';
  let inQuotes = false;
  
  for (let i = 0; i < line.length; i++) {
    const char = line[i];
    
    if (char === '"') {
      inQuotes = !inQuotes;
    } else if (char === ',' && !inQuotes) {
      parts.push(current.trim());
      current = '';
    } else {
      current += char;
    }
  }
  
  parts.push(current.trim());
  return parts;
};

// 辅助函数：解析分隔符并分割行
const parseLine = (line) => {
  // 智能处理多种分隔符格式
  let parts;
  
  // 首先检查是否包含逗号或制表符（CSV和TSV格式）
  if (line.includes(',')) {
    // CSV格式：用逗号分隔，处理可能存在的引号
    parts = parseCsvLine(line);
  } else if (line.includes('\t')) {
    // TSV格式：用制表符分隔
    parts = line.split('\t').map(part => part.trim());
  } else {
    // 空格分隔格式：支持一个或多个空格
    parts = line.split(/\s+/).filter(part => part !== '');
  }
  
  // 过滤空元素
  return parts.filter(part => part !== '');
};

// 辅助函数：检测是否为表头行
const isHeaderLine = (line, headerKeywords) => {
  // 优先检测表头关键词
  const lowerLine = line.toLowerCase();
  for (const keyword of headerKeywords) {
    if (lowerLine.includes(keyword)) {
      return true;
    }
  }
  
  // 如果没有关键词，尝试通过数据类型分析来判断
  const headerParts = parseLine(line);
  
  // 检查分割后的部分是否大多数都不是数字（表头通常是文本）
  if (headerParts.length >= 2) {
    const nonNumericCount = headerParts.filter(part => !isNumeric(part.trim())).length;
    // 如果超过一半的部分不是数字，很可能是表头
    return nonNumericCount > headerParts.length / 2;
  }
  
  return false;
};

// 打开文件对话框
const openFileDialog = async (fileType) => {
  const selected = await open({
    multiple: false,
    filters: [
      { name: fileType, extensions: ['txt', 'csv', 'data', 'dat'] }
    ]
  });
  
  if (selected) {
    // 根据文件类型设置对应的文件路径
    if (fileType === 'c.data') {
      cFilePath.value = selected;
    } else if (fileType === 'data.txt') {
      dataFilePath.value = selected;
    }
  }
};

// 加载并解析高程数据文件用于预览
const loadPreviewData = async (filePath) => {
  if (!filePath) {
    previewData.value = [];
    previewError.value = '';
    return;
  }
  
  isLoadingPreview.value = true;
  previewError.value = '';
  
  try {
    const content = await readTextFile(filePath);
    
    // 检查内容是否为空
    if (!content || content.trim() === '') {
      previewError.value = '文件内容为空';
      previewData.value = [];
      return;
    }
    
    // 支持多种换行符格式
    const lines = content.split(/\r?\n/);
    const parsedData = [];
    let hasHeader = false;
    
    // 最多预览前50行数据
    const maxLines = Math.min(lines.length, 50);
    
    for (let i = 0; i < maxLines; i++) {
      let line = lines[i].trim();
      
      // 跳过空行、注释行
      if (!line || line.startsWith('#') || line.startsWith(';') || line.startsWith('//')) {
        continue;
      }
      
      // 智能识别表头
      if (i === 0) {
        const headerKeywords = ['桩号', '高程', 'stake', 'elevation', 'point', 'height'];
        hasHeader = isHeaderLine(line, headerKeywords);
        if (hasHeader) continue;
      }
      
      const parts = parseLine(line);
      
      // 检查是否成功分割出至少两个非空部分，且包含有效的高程值
      if (parts.length >= 2 && parts[0]) {
        // 支持两种格式：
        // 1. 两列格式：桩号, 高程
        // 2. 四列格式：桩号, X, Y, 高程
        if (parts.length === 2 && isNumeric(parts[1])) {
          parsedData.push({
            index: hasHeader ? i : i + 1,
            stake: parts[0],
            elevation: parts[1],
            x: '', // 两列格式时X为空
            y: ''  // 两列格式时Y为空
          });
        } else if (parts.length >= 4 && isNumeric(parts[3])) {
          parsedData.push({
            index: hasHeader ? i : i + 1,
            stake: parts[0],
            x: parts[1],
            y: parts[2],
            elevation: parts[3]
          });
        }
      }
    }
    
    previewData.value = parsedData;
    
    // 如果没有解析到任何数据，但文件不为空，给出更详细的错误信息
    if (parsedData.length === 0) {
      previewError.value = '未找到有效的高程数据。支持的格式包括：\n1. 逗号分隔(CSV)\n2. 空格分隔\n3. 制表符分隔\n支持两种数据格式：\n- 两列格式：桩号, 高程\n- 四列格式：桩号, X, Y, 高程';
    }
  } catch (error) {
    previewError.value = `预览失败: ${error}`;
    previewData.value = [];
  } finally {
    isLoadingPreview.value = false;
  }
};

// 加载并解析待处理数据文件用于预览
const loadDataPreview = async (filePath) => {
  if (!filePath) {
    dataPreviewData.value = [];
    dataPreviewError.value = '';
    return;
  }
  
  isLoadingDataPreview.value = true;
  dataPreviewError.value = '';
  
  try {
    const content = await readTextFile(filePath);
    
    // 检查内容是否为空
    if (!content || content.trim() === '') {
      dataPreviewError.value = '文件内容为空';
      dataPreviewData.value = [];
      return;
    }
    
    // 支持多种换行符格式
    const lines = content.split(/\r?\n/);
    const parsedData = [];
    let hasHeader = false;
    
    // 最多预览前50行数据
    const maxLines = Math.min(lines.length, 50);
    
    for (let i = 0; i < maxLines; i++) {
      let line = lines[i].trim();
      
      // 跳过空行、注释行
      if (!line || line.startsWith('#') || line.startsWith(';') || line.startsWith('//')) {
        continue;
      }
      
      // 智能识别表头
      if (i === 0) {
        const headerKeywords = ['桩号', 'z', 'data', 'stake', 'point'];
        hasHeader = isHeaderLine(line, headerKeywords);
        if (hasHeader) continue;
      }
      
      const parts = parseLine(line);
      
      // 检查是否成功分割出至少两个非空部分
      if (parts.length >= 2 && parts[0]) {
        // 支持三列格式：桩号, Z, data
        if (parts.length >= 3) {
          parsedData.push({
            index: hasHeader ? i : i + 1,
            stake: parts[0],
            z: parts[1],
            data: parts[2]
          });
        } else if (parts.length === 2) {
          // 支持两列格式：桩号, 数据
          parsedData.push({
            index: hasHeader ? i : i + 1,
            stake: parts[0],
            z: '', // 两列格式时Z为空
            data: parts[1]
          });
        }
      }
    }
    
    dataPreviewData.value = parsedData;
    
    // 如果没有解析到任何数据，但文件不为空，给出更详细的错误信息
    if (parsedData.length === 0) {
      dataPreviewError.value = '未找到有效的待处理数据。支持的格式包括：\n1. 逗号分隔(CSV)\n2. 空格分隔\n3. 制表符分隔\n支持两种数据格式：\n- 两列格式：桩号, 数据\n- 三列格式：桩号, Z, 数据';
    }
  } catch (error) {
    dataPreviewError.value = `预览失败: ${error}`;
    dataPreviewData.value = [];
  } finally {
    isLoadingDataPreview.value = false;
  }
};

// 监听文件路径变化，自动加载预览数据
watch(cFilePath, (newPath) => {
  loadPreviewData(newPath);
}, { immediate: true });

watch(dataFilePath, (newPath) => {
  loadDataPreview(newPath);
}, { immediate: true });

// 执行线性回归
const performLinearRegression = (data) => {
  const n = data.length;
  let sumX = 0, sumY = 0, sumXY = 0, sumX2 = 0;

  for (let i = 0; i < n; i++) {
    const x = data[i].stake;
    const y = data[i].elevation;
    sumX += x;
    sumY += y;
    sumXY += x * y;
    sumX2 += x * x;
  }

  const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
  const intercept = (sumY - slope * sumX) / n;

  return {
    slope,
    intercept,
    predict: function(x) {
      return slope * x + intercept;
    }
  };
};

// 通用数据文件解析函数
const parseDataFile = async (filePath, isFourColumn = false) => {
  const content = await readTextFile(filePath);
  const lines = content.split(/\r?\n/);
  const data = [];
  let hasHeader = false;
  
  const headerKeywords = isFourColumn ? 
    ['桩号', '高程', 'stake', 'elevation', 'x', 'y'] : 
    ['桩号', '高程', 'stake', 'elevation'];

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i].trim();
    
    // 跳过空行、注释行
    if (!line || line.startsWith('#') || line.startsWith(';') || line.startsWith('//')) {
      continue;
    }

    // 智能识别表头
    if (i === 0) {
      hasHeader = isHeaderLine(line, headerKeywords);
      if (hasHeader) continue;
    }

    const parts = parseLine(line);

    // 提取数据
    if (isFourColumn) {
      // 四列格式：桩号, X, Y, 高程
      if (parts.length >= 4 && isNumeric(parts[0]) && isNumeric(parts[1]) && isNumeric(parts[2]) && isNumeric(parts[3])) {
        data.push({
          stake: parseFloat(parts[0]),
          x: parseFloat(parts[1]),
          y: parseFloat(parts[2]),
          elevation: parseFloat(parts[3])
        });
      }
    } else {
      // 两列或四列中的桩号和高程
      if (parts.length >= 2 && isNumeric(parts[0]) && isNumeric(parts[1])) {
        data.push({
          stake: parseFloat(parts[0]),
          elevation: parseFloat(parts[1])
        });
      } else if (parts.length >= 4 && isNumeric(parts[0]) && isNumeric(parts[3])) {
        data.push({
          stake: parseFloat(parts[0]),
          elevation: parseFloat(parts[3])
        });
      }
    }
  }

  return data;
};

// 处理待插值数据
const processDataForInterpolation = (content, models, includeXY = false) => {
  const lines = content.split(/\r?\n/);
  const processedData = [];
  let hasHeader = false;
  const headerKeywords = ['桩号', 'z', 'data', 'stake'];

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i].trim();
    
    // 跳过空行、注释行
    if (!line || line.startsWith('#') || line.startsWith(';') || line.startsWith('//')) {
      continue;
    }

    // 智能识别表头
    if (i === 0) {
      hasHeader = isHeaderLine(line, headerKeywords);
      if (hasHeader) continue;
    }

    const parts = parseLine(line);

    // 提取数据并计算
    if (parts.length >= 2 && isNumeric(parts[0])) {
      const stake = parseFloat(parts[0]);
      const z = parts.length >= 3 && isNumeric(parts[1]) ? parseFloat(parts[1]) : 0;
      const data = parts.length >= 3 ? parts[2] : (parts.length >= 2 ? parts[1] : '');
      
      // 过滤data值大于1000000的记录
      if (isNumeric(data) && parseFloat(data) > 1000000) {
        continue;
      }
      
      // 根据是否需要包含XY坐标选择不同的处理逻辑
      if (includeXY && models.xModel && models.yModel && models.elevationModel) {
        // 使用线性模型预测X、Y和高程
        const predictedX = models.xModel.predict(stake);
        const predictedY = models.yModel.predict(stake);
        const predictedElevation = models.elevationModel.predict(stake);
        
        // 计算最终高程：预测高程 + 数据文件中的z值
        const finalElevation = predictedElevation + z;
        
        processedData.push({
          stake: parts[0], // 保留原始桩号格式
          x: predictedX.toFixed(6), // 保留6位小数
          y: predictedY.toFixed(6), // 保留6位小数
          elevation: finalElevation.toFixed(6), // 保留6位小数
          data: data
        });
      } else if (models.linearModel) {
        // 使用线性模型预测高程
        const predictedElevation = models.linearModel.predict(stake);
        // 计算最终高程：预测高程 + 数据文件中的z值
        const finalElevation = predictedElevation + z;
        
        processedData.push({
          stake: parts[0], // 保留原始桩号格式
          elevation: finalElevation.toFixed(6), // 保留6位小数
          data: data
        });
      }
    }
  }

  return processedData;
};

// 执行插值高程操作（输出桩号、高程、data）
const performInterpolation = async () => {
  if (!cFilePath.value || !dataFilePath.value) {
    processMessage.value = '请先选择高程数据文件和待处理数据文件';
    return;
  }

  isProcessing.value = true;
  processMessage.value = '正在进行插值计算...';

  try {
    // 1. 读取高程数据并进行线性拟合
    const elevationData = await parseDataFile(cFilePath.value);
    if (elevationData.length < 2) {
      processMessage.value = '高程数据点不足，无法进行线性拟合';
      isProcessing.value = false;
      return;
    }

    // 2. 执行线性拟合
    const linearModel = performLinearRegression(elevationData);
    
    // 3. 读取待处理数据并处理
    const dataFileContent = await readTextFile(dataFilePath.value);
    const processedData = processDataForInterpolation(dataFileContent, { linearModel });
    
    // 4. 保存结果到CSV文件
    const savePath = await save({
      filters: [
        { name: 'CSV文件', extensions: ['csv'] }
      ]
    });

    if (savePath) {
      // 生成CSV内容
      let csvContent = '桩号,高程,data\n';
      processedData.forEach(item => {
        csvContent += `${item.stake},${item.elevation},${item.data}\n`;
      });

      // 写入文件
      await writeTextFile(savePath, csvContent);
      processMessage.value = `处理完成！结果已保存至：${savePath}`;
    } else {
      processMessage.value = '操作已取消';
    }
  } catch (error) {
    processMessage.value = `处理失败: ${error}`;
    console.error('处理失败:', error);
  } finally {
    isProcessing.value = false;
  }
};

// 执行四列高程数据插值（输出桩号、X、Y、高程、DATA）
const performInterpolationWithXY = async () => {
  if (!cFilePath.value || !dataFilePath.value) {
    processMessage.value = '请先选择高程数据文件和待处理数据文件';
    return;
  }

  isProcessing.value = true;
  processMessage.value = '正在进行插值计算（带X、Y坐标）...';

  try {
    // 1. 读取四列格式的高程数据（桩号、X、Y、高程）
    const elevationData = await parseDataFile(cFilePath.value, true);
    if (elevationData.length < 2) {
      processMessage.value = '高程数据点不足或不是四列格式，无法进行插值';
      isProcessing.value = false;
      return;
    }

    // 2. 对X、Y和高程分别进行线性拟合
    const xModel = performLinearRegression(elevationData.map(item => ({ stake: item.stake, elevation: item.x })));
    const yModel = performLinearRegression(elevationData.map(item => ({ stake: item.stake, elevation: item.y })));
    const elevationModel = performLinearRegression(elevationData.map(item => ({ stake: item.stake, elevation: item.elevation })));
    
    // 3. 读取待处理数据并处理
    const dataFileContent = await readTextFile(dataFilePath.value);
    const processedData = processDataForInterpolation(dataFileContent, 
      { xModel, yModel, elevationModel }, true);
    
    // 4. 保存结果到CSV文件
    const savePath = await save({
      filters: [
        { name: 'CSV文件', extensions: ['csv'] }
      ]
    });

    if (savePath) {
      // 生成CSV内容
      let csvContent = '桩号,X,Y,高程,DATA\n';
      processedData.forEach(item => {
        csvContent += `${item.stake},${item.x},${item.y},${item.elevation},${item.data}\n`;
      });

      // 写入文件
      await writeTextFile(savePath, csvContent);
      processMessage.value = `处理完成！结果已保存至：${savePath}`;
    } else {
      processMessage.value = '操作已取消';
    }
  } catch (error) {
    processMessage.value = `处理失败: ${error}`;
    console.error('处理失败:', error);
  } finally {
    isProcessing.value = false;
  }
};
</script>

<template>
  <main class="container">
    <h1>高程点插值</h1>
    
    <div class="file-selection">
      <h2>文件选择</h2>
      
      <div class="file-input-group">
        <label>高程数据（桩号，高程）:</label>
        <div class="file-input-row">
          <input type="text" v-model="cFilePath" readonly />
          <button @click="openFileDialog('c.data')">
            选择文件
          </button>
        </div>
        
        <!-- c.data文件预览区域 -->
        <div v-if="cFilePath" class="file-preview">
          <h3>文件预览</h3>
          <div v-if="isLoadingPreview" class="preview-loading">
            <div class="loading-spinner small"></div>
            <span>正在加载文件内容...</span>
          </div>
          <div v-else-if="previewError" class="preview-error">
            {{ previewError }}
          </div>
          <div v-else-if="previewData.length > 0" class="preview-table-container">
            <table class="preview-table">
              <thead>
                <tr>
                  <th>行号</th>
                  <th>桩号</th>
                  <th>X坐标</th>
                  <th>Y坐标</th>
                  <th>高程</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in previewData" :key="item.index">
                  <td>{{ item.index }}</td>
                  <td>{{ item.stake }}</td>
                  <td>{{ item.x || '-' }}</td>
                  <td>{{ item.y || '-' }}</td>
                  <td>{{ item.elevation }}</td>
                </tr>
              </tbody>
            </table>
            <div v-if="previewData.length >= 50" class="preview-note">
              仅显示前50行数据
            </div>
          </div>
        </div>
      </div>
      
      <div class="file-input-group">
        <label>数据(桩号，Z，data):</label>
        <div class="file-input-row">
          <input type="text" v-model="dataFilePath" readonly />
          <button @click="openFileDialog('data.txt')">
            选择文件
          </button>
        </div>
        
        <!-- 数据文件预览区域 -->
        <div v-if="dataFilePath" class="file-preview">
          <h3>文件预览</h3>
          <div v-if="isLoadingDataPreview" class="preview-loading">
            <div class="loading-spinner small"></div>
            <span>正在加载文件内容...</span>
          </div>
          <div v-else-if="dataPreviewError" class="preview-error">
            {{ dataPreviewError }}
          </div>
          <div v-else-if="dataPreviewData.length > 0" class="preview-table-container">
            <table class="preview-table">
              <thead>
                <tr>
                  <th>行号</th>
                  <th>桩号</th>
                  <th>Z</th>
                  <th>数据</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in dataPreviewData" :key="item.index">
                  <td>{{ item.index }}</td>
                  <td>{{ item.stake }}</td>
                  <td>{{ item.z || '-' }}</td>
                  <td>{{ item.data }}</td>
                </tr>
              </tbody>
            </table>
            <div v-if="dataPreviewData.length >= 50" class="preview-note">
              仅显示前50行数据
            </div>
          </div>
        </div>
      </div>
      
    </div>
    
    <!-- 操作按钮区域 -->
    <div class="operations">
      <h2>操作</h2>
      <div class="operation-buttons">
        <button @click="performInterpolation" :disabled="isProcessing">
          {{ isProcessing ? '处理中...' : '执行插值 (桩号,高程,data)' }}
        </button>
        <button @click="performInterpolationWithXY" :disabled="isProcessing">
          {{ isProcessing ? '处理中...' : '执行插值 (桩号,X,Y,高程,DATA)' }}
        </button>
        <div v-if="processMessage" class="process-message">
          {{ processMessage }}
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.file-selection,
.operations {
  margin: 20px 0;
  padding: 20px;
  background-color: #ffffff;
  border-radius: 10px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.file-input-group {
  margin: 15px 0;
  text-align: left;
}

.file-input-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
  color: #333;
}

.file-input-row {
  display: flex;
  gap: 10px;
  align-items: center;
}

.file-input-row input {
  flex: 1;
  background-color: #f5f5f5;
  cursor: not-allowed;
}

/* 文件预览样式 */
.file-preview {
  margin-top: 15px;
  padding: 15px;
  background-color: #fafafa;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
}

.file-preview h3 {
  margin-top: 0;
  margin-bottom: 10px;
  font-size: 14px;
  color: #666;
}

.preview-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #666;
  font-size: 14px;
}

.loading-spinner.small {
  width: 16px;
  height: 16px;
  border-width: 2px;
}

.preview-error {
  color: #dc143c;
  background-color: #ffe4e1;
  padding: 10px;
  border-radius: 4px;
  font-size: 14px;
}

.preview-table-container {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
}

.preview-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.preview-table th,
.preview-table td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid #e0e0e0;
}

.preview-table th {
  background-color: #f8f9fa;
  font-weight: 600;
  color: #333;
  position: sticky;
  top: 0;
  z-index: 10;
}

.preview-table tr:hover {
  background-color: #f5f5f5;
}

.preview-note {
  margin-top: 10px;
  font-size: 12px;
  color: #999;
  text-align: right;
}

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: 0;
  min-height: 100vh;
}

.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 40px 20px;
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

h1 {
  text-align: center;
  color: #333;
  margin-bottom: 30px;
  font-size: 28px;
}

h2 {
  text-align: center;
  color: #555;
  margin-bottom: 20px;
  font-size: 20px;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover:not(:disabled) {
  border-color: #396cd8;
}
button:active:not(:disabled) {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

input,
button {
  outline: none;
}

.operation-buttons {
  text-align: center;
  margin-top: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
}

.operation-buttons button {
  min-width: 250px;
}

.process-message {
  margin-top: 15px;
  padding: 10px;
  border-radius: 4px;
  background-color: #f8f9fa;
  color: #333;
  font-size: 14px;
  line-height: 1.5;
  word-wrap: break-word;
}

input:focus {
  border-color: #646cff;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  .file-selection,
  .operations {
    background-color: #3a3a3a;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  }

  .file-input-group label {
    color: #f6f6f6;
  }

  .file-input-row input {
    background-color: #4a4a4a;
    color: #f6f6f6;
  }

  h1, h2 {
    color: #f6f6f6;
  }
  
  .process-message {
    background-color: #4a4a4a;
    color: #f6f6f6;
  }
}
</style>
