.PHONY: all
all: build-console-dist build-music-dist


.PHONY: build-console-dist
build-console-dist:
	@echo "Building console..."
	rm -rf console-dist
# 跳转到../vue-console目录下，使用npm run build命令打包，打包后的文件在../vue-console/dist目录下，将其中的文件拷贝到../console-dist目录下，然后回到当前目录
	cd ../vue-console \
		&& npm run build \
		&& cd - \
		&& cp -r ../vue-console/dist ./console-dist
# 删除*.map文件
	rm -rf ./console-dist/js/*.map
# 对所有的html, js, css文件进行gzip压缩
	@for file in $$(find ./console-dist -type f -name "*.html" -o -name "*.js" -o -name "*.css"); do \
		echo "Compressing $$file"; \
		gzip -9 -c "$$file" > "$$file.gz"; \
		rm -f "$$file"; \
		mv "$$file.gz" "$$file"; \
		echo "GZip compress success: $$file"; \
	done
	
	@echo "Build console success!"

.PHONY: build-console-dist
build-music-dist:
	@echo "Building music..."
	@mkdir -p ./music-dist
# 将./music下的每个mid文件，使用python3 tools/merge_midi.py分别合并成一个个单音轨mid文件，输出目录为./music-dist
# tools/merge_midi.py <input.mid> <output.mid>
	@for file in ./music/*.mid; do \
		echo "Merging $$file"; \
		output_file=./music-dist/$$(basename "$$file"); \
		python3 tools/merge_midi.py "$$file" "$$output_file"; \
		echo "Merge success output: $$output_file"; \
	done
