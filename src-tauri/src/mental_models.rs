use rusqlite::OptionalExtension;
use serde_json::{json, Value};

pub const CATEGORIES: &[(&str, &str)] = &[
    ("problem-cognition", "问题认知"),
    ("risk-error", "风险与错误"),
    ("decision-judgment", "决策判断"),
    ("strategy", "战略"),
    ("customer-product", "客户与产品"),
    ("growth", "增长"),
    ("organization", "组织"),
    ("learning-evolution", "学习与进化"),
];

#[derive(Clone, Copy)]
pub struct Seed {
    pub slug: &'static str,
    pub name: &'static str,
    pub category_id: &'static str,
    pub definition: &'static str,
    pub core_idea: &'static str,
    pub source_person: &'static str,
    pub source_theory: &'static str,
    pub tags: &'static str,
    pub triggers: &'static str,
    pub output: &'static str,
}

const fn seed(
    slug: &'static str,
    name: &'static str,
    category_id: &'static str,
    definition: &'static str,
    core_idea: &'static str,
    source_person: &'static str,
    source_theory: &'static str,
    tags: &'static str,
    triggers: &'static str,
    output: &'static str,
) -> Seed {
    Seed {
        slug,
        name,
        category_id,
        definition,
        core_idea,
        source_person,
        source_theory,
        tags,
        triggers,
        output,
    }
}

pub const SEEDS: &[Seed] = &[
    seed(
        "first-principles",
        "第一性原理",
        "problem-cognition",
        "把复杂问题拆回不可再分的事实与约束。",
        "不从惯例出发，从事实重新推导。",
        "Elon Musk",
        "First Principles",
        "CEO,认知,产品,创新",
        "复杂问题,新产品,行业变化",
        "事实清单\n关键约束\n重新推导的方案",
    ),
    seed(
        "circle-of-competence",
        "能力圈",
        "problem-cognition",
        "只在理解边界和优势范围内做重要判断。",
        "知道自己知道什么，也知道自己不知道什么。",
        "Charlie Munger",
        "Circle of Competence",
        "CEO,战略,风险",
        "陌生市场,重大投入,能力不足",
        "能力边界\n缺口\n是否继续进入",
    ),
    seed(
        "base-rate",
        "基准率思维",
        "problem-cognition",
        "先看同类事件的历史概率，再评估个案。",
        "个案叙事不能替代基础概率。",
        "Daniel Kahneman",
        "Base Rate",
        "概率,风险,投资",
        "预测,新项目,成功率判断",
        "参考样本\n基准概率\n个案修正",
    ),
    seed(
        "multiple-mental-models",
        "多元思维模型",
        "problem-cognition",
        "用多个学科视角交叉检查同一问题。",
        "单一模型容易造成盲区，组合模型可以提高判断质量。",
        "Charlie Munger",
        "Latticework of Mental Models",
        "CEO,系统,决策",
        "重大决策,跨部门问题,复杂系统",
        "调用模型\n共同结论\n冲突与盲区",
    ),
    seed(
        "inversion",
        "逆向思维",
        "risk-error",
        "从目标的失败结果反向推导原因与预防措施。",
        "先避免明显错误，再追求成功。",
        "Charlie Munger",
        "Inversion",
        "风险,CEO,投资",
        "重大决策,高风险项目,大额投入",
        "失败原因\n致命风险\n风险等级\n预防措施",
    ),
    seed(
        "cognitive-bias",
        "认知偏差",
        "risk-error",
        "识别判断中系统性的心理偏误。",
        "先检查自己的判断机制，再相信结论。",
        "Daniel Kahneman",
        "Thinking, Fast and Slow",
        "风险,组织,决策",
        "情绪化判断,冲突,过度自信",
        "偏差类型\n证据\n纠偏动作",
    ),
    seed(
        "premortem",
        "预演失败",
        "risk-error",
        "假设方案已经失败，提前寻找可能原因。",
        "在行动前暴露失败路径。",
        "Gary Klein",
        "Premortem",
        "风险,项目,执行",
        "项目启动,投资,战略调整",
        "失败假设\n原因\n预防实验",
    ),
    seed(
        "margin-of-safety",
        "安全边际",
        "risk-error",
        "让判断和投入保留足够缓冲，降低估计错误的代价。",
        "不要只在最乐观情景下成立。",
        "Benjamin Graham",
        "Margin of Safety",
        "投资,风险,财务",
        "大额投入,现金流,高不确定性",
        "基准情景\n最坏情景\n缓冲空间",
    ),
    seed(
        "second-order-thinking",
        "二阶思维",
        "risk-error",
        "分析行动直接后果之后的连锁影响。",
        "问“然后会怎样”，避免只看第一步。",
        "Howard Marks",
        "Second-Order Thinking",
        "战略,风险,系统",
        "竞争变化,降价,组织调整",
        "一阶结果\n二阶结果\n长期副作用",
    ),
    seed(
        "probabilistic-thinking",
        "概率思维",
        "decision-judgment",
        "用概率、区间和更新而不是绝对断言表达不确定性。",
        "判断不是预测单点，而是管理概率分布。",
        "Annie Duke",
        "Thinking in Bets",
        "概率,决策,投资",
        "不确定性,预测,资源下注",
        "概率区间\n证据\n更新后的置信度",
    ),
    seed(
        "opportunity-cost",
        "机会成本",
        "decision-judgment",
        "比较选择一个方案所放弃的最佳替代方案。",
        "资源有限，真正的成本是放弃的机会。",
        "Friedrich von Wieser",
        "Opportunity Cost",
        "CEO,资源,战略",
        "多项目竞争,预算,时间分配",
        "候选方案\n最佳替代\n放弃代价",
    ),
    seed(
        "reversible-irreversible",
        "可逆/不可逆决策",
        "decision-judgment",
        "根据决策可逆程度选择行动速度与审慎程度。",
        "可逆决策快速试错，不可逆决策提高证据门槛。",
        "Jeff Bezos",
        "Type 1 / Type 2 Decisions",
        "CEO,执行,风险",
        "重大决策,实验,大额投入",
        "可逆性\n证据门槛\n下一步",
    ),
    seed(
        "hypothesis-driven-decision",
        "假设驱动决策",
        "decision-judgment",
        "把商业决策拆成可验证的假设与下注。",
        "假设、验证、更新概率，再决定是否行动。",
        "Decision Science",
        "Hypothesis-Driven Decision",
        "决策,实验,概率",
        "新产品,新市场,高不确定性",
        "假设
验证方式
验证成本
验证结果
更新概率",
    ),
    seed(
        "information-value",
        "信息价值",
        "decision-judgment",
        "评估获取信息是否值得其成本。",
        "只有可能改变决策的信息才值得调查。",
        "Decision Analysis",
        "Value of Information",
        "决策,调查,效率",
        "数据缺口,调研,等待",
        "信息成本\n可能改变的决策\n调查或行动",
    ),
    seed(
        "stopping-rule",
        "停止规则",
        "decision-judgment",
        "在行动前定义何时继续、观察、重评估或停止。",
        "沉没成本不能成为继续投入的理由。",
        "Decision Theory",
        "Stopping Rule",
        "项目,风险,执行",
        "项目投入,实验,现金流压力",
        "停止条件\n当前指标\n动作",
    ),
    seed(
        "strategy-kernel",
        "战略内核",
        "strategy",
        "用诊断、指导方针和连贯行动形成战略。",
        "战略不是目标口号，而是针对障碍的选择。",
        "Richard Rumelt",
        "Good Strategy Bad Strategy",
        "战略,CEO,竞争",
        "战略规划,业务转型,资源取舍",
        "诊断\n指导方针\n连贯行动",
    ),
    seed(
        "five-forces",
        "五力模型",
        "strategy",
        "分析行业竞争结构与利润压力。",
        "利润池由竞争力量共同决定。",
        "Michael Porter",
        "Five Forces",
        "战略,市场,竞争",
        "进入市场,竞品降价,行业变化",
        "五种力量\n利润压力\n战略位置",
    ),
    seed(
        "value-chain",
        "价值链",
        "strategy",
        "拆解企业创造价值与产生成本的活动。",
        "竞争优势来自价值活动之间的系统配置。",
        "Michael Porter",
        "Value Chain",
        "战略,运营,产品",
        "成本优化,供应链,差异化",
        "价值活动\n成本来源\n优势环节",
    ),
    seed(
        "strategic-inflection",
        "战略拐点",
        "strategy",
        "识别外部变化使原有战略假设失效的时刻。",
        "真正危险的是变化发生而组织仍按旧逻辑行动。",
        "Andy Grove",
        "Strategic Inflection Point",
        "战略,变化,AI",
        "行业变化,技术冲击,竞争重构",
        "变化信号\n失效假设\n转向选项",
    ),
    seed(
        "innovators-dilemma",
        "创新者窘境",
        "strategy",
        "解释成功企业为何可能被新技术和新市场颠覆。",
        "既有客户与利润结构可能阻碍下一代创新。",
        "Clayton Christensen",
        "Disruptive Innovation",
        "战略,创新,产品",
        "新技术,低端竞争,业务转型",
        "现有优势\n颠覆路径\n双轨行动",
    ),
    seed(
        "jtbd",
        "Jobs to Be Done",
        "customer-product",
        "从客户要完成的任务理解需求与产品选择。",
        "客户购买的不是产品，而是完成任务的进展。",
        "Clayton Christensen",
        "Jobs to Be Done",
        "产品,客户,增长",
        "新产品,用户流失,需求不清",
        "客户任务\n触发情境\n替代方案",
    ),
    seed(
        "customer-obsession",
        "客户至上",
        "customer-product",
        "以客户长期价值和真实反馈作为优先判断依据。",
        "客户问题比内部假设更接近产品真相。",
        "Jeff Bezos",
        "Customer Obsession",
        "客户,产品,CEO",
        "产品取舍,投诉,增长",
        "客户问题\n证据\n优先行动",
    ),
    seed(
        "feedback-loop",
        "反馈回路",
        "customer-product",
        "识别行动、结果与反馈之间的循环关系。",
        "系统会放大或抑制行为，关键在于回路结构。",
        "Systems Thinking",
        "Feedback Loop",
        "产品,增长,组织",
        "指标变化,重复问题,迭代",
        "回路节点\n增强或平衡\n干预点",
    ),
    seed(
        "leverage",
        "杠杆",
        "growth",
        "用较少资源放大产出、影响或速度。",
        "优先寻找可复制、可放大的约束突破点。",
        "Archimedes",
        "Leverage",
        "增长,效率,CEO",
        "资源有限,规模化,效率",
        "杠杆点\n投入产出\n放大风险",
    ),
    seed(
        "long-termism",
        "长期主义",
        "learning-evolution",
        "用长期目标和累积效应约束短期选择。",
        "不被短期噪音带偏，持续建设可复利的能力。",
        "Jeff Bezos",
        "Long-term Thinking",
        "长期主义,战略,增长",
        "行业变化,长期投资,能力建设",
        "长期目标\n短期代价\n累积优势",
    ),
    seed(
        "compound-interest",
        "复利",
        "growth",
        "小幅持续改善通过时间积累形成巨大差异。",
        "增长率和持续时间共同决定长期结果。",
        "Warren Buffett",
        "Compounding",
        "增长,长期主义,投资",
        "长期项目,能力建设,品牌",
        "起点\n增长率\n时间\n阻断因素",
    ),
    seed(
        "flywheel",
        "飞轮效应",
        "growth",
        "多个相互强化的活动形成自我加速循环。",
        "增长来自环环相扣，而非单点动作。",
        "Jeff Bezos",
        "Flywheel",
        "增长,平台,客户",
        "增长停滞,平台,复购",
        "飞轮环节\n阻力\n启动动作",
    ),
    seed(
        "network-effects",
        "网络效应",
        "growth",
        "参与者增加提升产品对其他参与者的价值。",
        "价值随网络规模和连接质量增长。",
        "Robert Metcalfe",
        "Network Effects",
        "增长,平台,产品",
        "平台,社区,双边市场",
        "网络类型\n临界规模\n负向效应",
    ),
    seed(
        "economies-of-scale",
        "规模经济",
        "growth",
        "规模扩大后单位成本下降或能力提升。",
        "规模必须转化为真实的单位经济改善。",
        "Economics",
        "Economies of Scale",
        "增长,成本,运营",
        "扩张,采购,产能",
        "固定成本\n单位成本\n规模风险",
    ),
    seed(
        "incentives",
        "激励机制",
        "organization",
        "分析奖励、惩罚和指标如何塑造行为。",
        "不要只听口号，要看激励结构。",
        "Charlie Munger",
        "Incentives",
        "组织,管理,风险",
        "离职,执行偏差,绩效",
        "激励来源\n实际行为\n副作用",
    ),
    seed(
        "effective-manager",
        "有效管理者",
        "organization",
        "通过贡献、优先级和决策提高组织有效性。",
        "管理者的产出是组织整体产出。",
        "Peter Drucker",
        "Effective Executive",
        "组织,管理,CEO",
        "团队协作,管理升级,效率",
        "贡献\n优先级\n组织动作",
    ),
    seed(
        "strengths-management",
        "优势管理",
        "organization",
        "围绕个人与团队优势设计角色和协作。",
        "把优势变成组织结果，而不是修补所有弱点。",
        "Peter Drucker",
        "Managing Strengths",
        "组织,人才,管理",
        "招聘,分工,核心员工",
        "优势\n角色匹配\n缺口补偿",
    ),
    seed(
        "principle-based-management",
        "原则化管理",
        "organization",
        "用清晰原则代替临时命令，保持判断一致性。",
        "原则让组织在复杂情境中自主决策。",
        "Ray Dalio",
        "Principles",
        "组织,管理,决策",
        "规模化,授权,冲突",
        "原则\n边界\n例外处理",
    ),
    seed(
        "retrospective",
        "复盘",
        "learning-evolution",
        "比较预期与实际，提炼下一次可改变的行动。",
        "经验只有进入下一次行动才产生价值。",
        "Learning Organization",
        "Retrospective",
        "学习,结果,执行",
        "项目结束,结果偏差,重复问题",
        "预期\n实际\n偏差\n下一步",
    ),
    seed(
        "hypothesis-update",
        "假设更新",
        "learning-evolution",
        "把决策视为假设，并根据验证结果更新概率。",
        "不确定性下的进步来自持续校准。",
        "Bayesian Thinking",
        "Hypothesis Updating",
        "学习,概率,产品",
        "实验,新证据,市场验证",
        "假设\n验证\n证据\n更新概率",
    ),
    seed(
        "experience-extraction",
        "经验提炼",
        "learning-evolution",
        "从具体事件中提炼可复用的经验边界。",
        "经验必须包含适用条件和限制。",
        "Learning Organization",
        "Experience Extraction",
        "学习,知识,CEO",
        "复盘,重复事件,培训",
        "事实\n模式\n适用边界",
    ),
    seed(
        "decision-feedback",
        "决策反馈",
        "learning-evolution",
        "把决策、执行、结果和复盘连接起来校准判断。",
        "决策质量必须用结果和过程共同评价。",
        "Decision Science",
        "Decision Feedback",
        "学习,决策,结果",
        "重大决策,结果发生,模型评价",
        "预测\n结果\n准确度\n模型效果",
    ),
];

pub fn category_label(category_id: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(id, _)| *id == category_id)
        .map(|(_, label)| *label)
        .unwrap_or("待归类")
}

pub fn seed_value(seed: Seed, id: String, now: &str) -> Value {
    json!({
        "id": id, "entity": "mentalModels", "name": seed.name, "slug": seed.slug,
        "categoryId": seed.category_id, "category": category_label(seed.category_id),
        "definition": seed.definition, "coreIdea": seed.core_idea, "corePrinciple": seed.core_idea,
        "applicationScenarios": seed.triggers, "useCases": seed.triggers,
        "keyQuestions": "请把事实、假设、证据和下一步分开。", "methodSteps": "1. 定义问题\n2. 检查证据\n3. 形成输出", "steps": "1. 定义问题\n2. 检查证据\n3. 形成输出",
        "outputStructure": seed.output, "outputTemplate": seed.output,
        "sourcePerson": seed.source_person, "source": seed.source_person, "sourceTheory": seed.source_theory,
        "triggerConditions": seed.triggers, "trigger": seed.triggers, "tags": seed.tags,
        "difficulty": "intermediate", "status": "active", "needsReview": false,
        "createdAt": now, "updatedAt": now
    })
}

const RELATION_SEEDS: &[(&str, &str, &str)] = &[
    ("inversion", "margin-of-safety", "complementary"),
    ("inversion", "probabilistic-thinking", "complementary"),
    ("inversion", "second-order-thinking", "related"),
    ("first-principles", "jtbd", "related"),
    ("compound-interest", "flywheel", "related"),
    ("flywheel", "network-effects", "related"),
    ("strategic-inflection", "innovators-dilemma", "related"),
    ("opportunity-cost", "information-value", "complementary"),
    ("hypothesis-driven-decision", "hypothesis-update", "related"),
    ("stopping-rule", "margin-of-safety", "complementary"),
    ("five-forces", "value-chain", "related"),
];

pub fn migrate(connection: &rusqlite::Connection, applied_at: &str) -> Result<(), String> {
    let already: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version=10)",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if already {
        return Ok(());
    }

    let mut existing = std::collections::HashMap::<String, String>::new();
    let rows = {
        let mut stmt = connection.prepare("SELECT id, data_json FROM records WHERE entity='mentalModels' AND deleted_at IS NULL").map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    for (id, raw) in rows {
        let mut data: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
        let object = data.as_object_mut().ok_or("思维模型记录结构无效")?;
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let old_category = object
            .get("category")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let category_id = object
            .get("categoryId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if category_id.is_empty() {
            let mapped = if old_category.contains("风险") {
                "risk-error"
            } else if old_category.contains("增长") {
                "growth"
            } else if old_category.contains("组织") {
                "organization"
            } else if old_category.contains("战略") {
                "strategy"
            } else if old_category.contains("产品") || old_category.contains("客户") {
                "customer-product"
            } else if old_category.contains("学习") || old_category.contains("复盘") {
                "learning-evolution"
            } else {
                "decision-judgment"
            };
            object.insert("categoryId".into(), Value::String(mapped.into()));
            object.insert(
                "category".into(),
                Value::String(category_label(mapped).into()),
            );
        }
        let aliases = [
            ("definition", "coreIdea"),
            ("corePrinciple", "coreIdea"),
            ("useCases", "applicationScenarios"),
            ("trigger", "triggerConditions"),
            ("steps", "methodSteps"),
            ("outputTemplate", "outputStructure"),
        ];
        for (target, source) in aliases {
            if object
                .get(target)
                .and_then(Value::as_str)
                .unwrap_or("")
                .is_empty()
            {
                if let Some(value) = object
                    .get(source)
                    .cloned()
                    .filter(|v| !v.as_str().unwrap_or("").is_empty())
                {
                    object.insert(target.into(), value);
                }
            }
        }
        object
            .entry("slug")
            .or_insert_with(|| Value::String(name.to_lowercase().replace(' ', "-")));
        let source_person = object
            .get("source")
            .cloned()
            .unwrap_or(Value::String("".into()));
        object.entry("sourcePerson").or_insert(source_person);
        object
            .entry("sourceTheory")
            .or_insert_with(|| Value::String("".into()));
        let trigger_conditions = object
            .get("trigger")
            .cloned()
            .unwrap_or(Value::String("".into()));
        object
            .entry("triggerConditions")
            .or_insert(trigger_conditions);
        object
            .entry("relatedModelIds")
            .or_insert_with(|| Value::Array(vec![]));
        object
            .entry("oppositeModelIds")
            .or_insert_with(|| Value::Array(vec![]));
        object
            .entry("difficulty")
            .or_insert_with(|| Value::String("intermediate".into()));
        object
            .entry("status")
            .or_insert_with(|| Value::String("active".into()));
        object.insert(
            "needsReview".into(),
            Value::Bool(
                object
                    .get("needsReview")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
            ),
        );
        let title = object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let slug = object
            .get("slug")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let body = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        connection
            .execute(
                "UPDATE records SET data_json=?2,title=?3,body=?4,updated_at=?5 WHERE id=?1",
                rusqlite::params![id, body, title, body, applied_at],
            )
            .map_err(|e| e.to_string())?;
        if !name.is_empty() {
            existing.insert(name, id.clone());
        }
        if !slug.is_empty() {
            existing.insert(slug, id.clone());
        }
    }

    for (index, seed) in SEEDS.iter().enumerate() {
        let found: Option<String> = connection.query_row("SELECT id FROM records WHERE entity='mentalModels' AND deleted_at IS NULL AND (json_extract(data_json,'$.slug')=?1 OR json_extract(data_json,'$.name')=?2) LIMIT 1", rusqlite::params![seed.slug, seed.name], |row| row.get(0)).optional().map_err(|e| e.to_string())?;
        let was_missing = found.is_none();
        let id = found.unwrap_or_else(|| format!("mentalModels-engine-{}-{}", applied_at, index));
        if was_missing {
            let data = seed_value(*seed, id.clone(), applied_at);
            let raw = serde_json::to_string(&data).map_err(|e| e.to_string())?;
            connection.execute("INSERT INTO records(id,entity,data_json,title,body,created_at,updated_at) VALUES(?1,'mentalModels',?2,?3,?4,?5,?5)", rusqlite::params![id, raw, seed.name, raw, applied_at]).map_err(|e| e.to_string())?;
        }
        existing.insert(seed.slug.into(), id.clone());
        existing.insert(seed.name.into(), id);
    }
    for (from_slug, to_slug, relation_type) in RELATION_SEEDS {
        let (Some(from), Some(to)) = (existing.get(*from_slug), existing.get(*to_slug)) else {
            continue;
        };
        connection.execute("INSERT OR IGNORE INTO relations(id,from_id,to_id,relation_type,created_at) VALUES(?1,?2,?3,?4,?5)", rusqlite::params![format!("mental-model-relation-{from_slug}-{to_slug}"), from, to, format!("mental_model:{relation_type}"), applied_at]).map_err(|e| e.to_string())?;
        let raw: String = connection
            .query_row(
                "SELECT data_json FROM records WHERE id=?1",
                rusqlite::params![from],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let mut data: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
        let object = data.as_object_mut().ok_or("思维模型关系记录结构无效")?;
        let key = if *relation_type == "opposite" {
            "oppositeModelIds"
        } else {
            "relatedModelIds"
        };
        let ids = object.entry(key).or_insert_with(|| Value::Array(vec![]));
        if let Some(array) = ids.as_array_mut() {
            if !array.iter().any(|v| v.as_str() == Some(to)) {
                array.push(Value::String(to.clone()));
            }
        }
        let body = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        connection
            .execute(
                "UPDATE records SET data_json=?2,body=?2,updated_at=?3 WHERE id=?1",
                rusqlite::params![from, body, applied_at],
            )
            .map_err(|e| e.to_string())?;
    }
    connection
        .execute(
            "INSERT OR IGNORE INTO schema_migrations(version,applied_at) VALUES(9,?1)",
            rusqlite::params![applied_at],
        )
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO schema_migrations(version,applied_at) VALUES(10,?1)",
            rusqlite::params![applied_at],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn migrate_followups(
    connection: &rusqlite::Connection,
    applied_at: &str,
) -> Result<(), String> {
    let already: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version=11)",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if already {
        return Ok(());
    }
    let find = |slug: &str| -> Result<Option<String>, String> {
        connection.query_row("SELECT id FROM records WHERE entity='mentalModels' AND deleted_at IS NULL AND json_extract(data_json,'$.slug')=?1 LIMIT 1", rusqlite::params![slug], |row| row.get(0)).optional().map_err(|error| error.to_string())
    };
    if let (Some(from), Some(to)) = (find("long-termism")?, find("stopping-rule")?) {
        connection.execute("INSERT OR IGNORE INTO relations(id,from_id,to_id,relation_type,created_at) VALUES(?1,?2,?3,'mental_model:opposite',?4)", rusqlite::params!["mental-model-relation-long-termism-stopping-rule", from, to, applied_at]).map_err(|e| e.to_string())?;
        let raw: String = connection
            .query_row(
                "SELECT data_json FROM records WHERE id=?1",
                rusqlite::params![from],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let mut data: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
        if let Some(object) = data.as_object_mut() {
            let ids = object
                .entry("oppositeModelIds")
                .or_insert_with(|| Value::Array(vec![]));
            if let Some(array) = ids.as_array_mut() {
                if !array.iter().any(|v| v.as_str() == Some(&to)) {
                    array.push(Value::String(to));
                }
            }
        }
        let body = serde_json::to_string(&data).map_err(|e| e.to_string())?;
        connection
            .execute(
                "UPDATE records SET data_json=?2,body=?2,updated_at=?3 WHERE id=?1",
                rusqlite::params![from, body, applied_at],
            )
            .map_err(|e| e.to_string())?;
    }
    connection
        .execute(
            "INSERT INTO schema_migrations(version,applied_at) VALUES(11,?1)",
            rusqlite::params![applied_at],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
