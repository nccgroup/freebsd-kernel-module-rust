OBJECTDIR?=target/objects

KMOD=hello
SRCS=hello.c
OBJS=$(OBJECTDIR)/*.o


.include<bsd.kmod.mk>
