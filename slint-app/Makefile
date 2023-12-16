build-console-dist:
	@echo "Building console..."
# 跳转到../vue-console目录下，使用npm run build命令打包，打包后的文件在../vue-console/dist目录下，将其中的文件拷贝到../console-dist目录下，然后回到当前目录
	cd ../vue-console \
		&& npm run build \
		&& cd - \
		&& cp -r ../vue-console/dist/* ./console-dist
# 删除*.map文件
	rm -rf ./console-dist/js/*.map
	@echo "Build console success!"

