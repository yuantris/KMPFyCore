#ifndef CORE_RS_H
#define CORE_RS_H
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
void core_rs_string_free(char* value);
double core_rs_eval(const char* expression);
double core_rs_eval_x(const char* expression, double x);
bool core_rs_contains_variable(const char* expression);
char* core_rs_calculate(const char* expression, int32_t mode);
char* core_rs_graph(const char* expression,double min_x,double max_x,double min_y,double max_y,int32_t samples,int32_t width,int32_t height);
char* core_rs_graph_polar(const char* expression,double min_x,double max_x,double min_y,double max_y,double min_t,double max_t,int32_t samples,int32_t width,int32_t height);
char* core_rs_graph_parametric(const char* x_expression,const char* y_expression,double min_x,double max_x,double min_y,double max_y,double min_t,double max_t,int32_t samples,int32_t width,int32_t height);
char* core_rs_analyze(const char* expression,double min_x,double max_x,double min_y,double max_y,int32_t samples,int32_t width,int32_t height);
uint64_t core_rs_search_create(void);
void core_rs_search_destroy(uint64_t handle);
bool core_rs_search_add(uint64_t handle,uint64_t id,const char* text);
bool core_rs_search_remove(uint64_t handle,uint64_t id);
void core_rs_search_clear(uint64_t handle);
size_t core_rs_search_size(uint64_t handle);
char* core_rs_search(uint64_t handle,const char* query,size_t limit);
uint64_t core_rs_zip_open(const char* path);
void core_rs_zip_close(uint64_t handle);
char* core_rs_zip_entries(uint64_t handle);
bool core_rs_zip_contains(uint64_t handle,const char* name);
bool core_rs_apk_is_apk(const char* path);
char* core_rs_apk_entries(const char* path);
#ifdef __cplusplus
}
#endif
#endif
