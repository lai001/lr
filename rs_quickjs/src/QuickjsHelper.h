#ifndef QuickjsHelper_H
#define QuickjsHelper_H

#ifdef __cplusplus
extern "C"
{
#endif

#include "quickjs.h"

    int QuickjsHelper_evalFile(JSContext *ctx, const char *filename, int module);

    int QuickjsHelper_evalBuffer(JSContext *ctx, const void *buffer, int bufferLength, const char *filename,
                                 int evalFlags);

    int QuickJS_ValueGetTag(JSValue v);

    void QuickJS_FreeValue(JSContext *ctx, JSValue v);

    void QuickJS_FreeValueRT(JSRuntime *rt, JSValue v);

    void QuickJS_DupValue(JSContext *ctx, JSValue v);

    JSValue QuickJS_DupValueRT(JSRuntime *rt, JSValueConst v);

    JSValue QuickJS_NewFloat64(JSContext *ctx, double d);

    JSValue QuickJS_NewInt32(JSContext *ctx, int32_t val);

    JSValue QuickJS_NewInt64(JSContext *ctx, int64_t val);

    JSValue QuickJS_NewBool(JSContext *ctx, JS_BOOL val);

    JS_BOOL QuickJS_VALUE_IS_NAN(JSValue v);

    double QuickJS_VALUE_GET_FLOAT64(JSValue v);

    int QuickJS_VALUE_GET_NORM_TAG(JSValue v);

    JS_BOOL QuickJS_IsNumber(JSValueConst v);

    JS_BOOL QuickJS_IsBigInt(JSContext *ctx, JSValueConst v);

    JS_BOOL QuickJS_IsBigFloat(JSValueConst v);

    JS_BOOL QuickJS_IsBigDecimal(JSValueConst v);

    JS_BOOL QuickJS_IsBool(JSValueConst v);

    JS_BOOL QuickJS_IsNull(JSValueConst v);

    JS_BOOL QuickJS_IsUndefined(JSValueConst v);

    JS_BOOL QuickJS_IsException(JSValueConst v);

    JS_BOOL QuickJS_IsUninitialized(JSValueConst v);

    JS_BOOL QuickJS_IsString(JSValueConst v);

    JS_BOOL QuickJS_IsSymbol(JSValueConst v);

    JS_BOOL QuickJS_IsObject(JSValueConst v);

    int QuickJS_ToUint32(JSContext *ctx, uint32_t *pres, JSValueConst val);

    int QuickJS_SetProperty(JSContext *ctx, JSValueConst this_obj, JSAtom prop, JSValue val);

    JSValue QuickJS_NewCFunction(JSContext *ctx, JSCFunction *func, const char *name, int length);

    JSValue QuickJS_NewCFunctionMagic(JSContext *ctx, JSCFunctionMagic *func, const char *name, int length,
                                      JSCFunctionEnum cproto, int magic);

    JSValue QuickJS_MKVAL(int tag, int val);

    JSValue QuickJS_NULL();

    JSValue QuickJS_UNDEFINED();

    JSValue QuickJS_FALSE();

    JSValue QuickJS_TRUE();

    JSValue QuickJS_EXCEPTION();

    JSValue QuickJS_UNINITIALIZED();

#ifdef __cplusplus
}
#endif

#endif